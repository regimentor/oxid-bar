use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{anyhow, bail};
use libpulse_binding::{
    callbacks::ListResult,
    context::{
        Context, FlagSet, State,
        introspect::Introspector,
        subscribe::{Facility, InterestMaskSet, Operation},
    },
    mainloop::standard::Mainloop,
    volume::{ChannelVolumes, Volume, VolumeLinear},
};

use crate::backend::pulse::{
    client::{AudioCmd, AudioEvent},
    output_info::OutputInfo,
};

type SharedIntrospector = Rc<RefCell<Introspector>>;
type ChannelCache = Rc<RefCell<HashMap<u32, u8>>>;

pub fn lesten_pulse_backend(
    cmd_rx: mpsc::Receiver<AudioCmd>,
    event_tx: mpsc::Sender<AudioEvent>,
) -> anyhow::Result<()> {
    let mut ml = Mainloop::new().ok_or_else(|| anyhow!("Failed to create PA mainloop"))?;

    let mut ctx = Context::new(&ml, "oxid-bar-audio")
        .ok_or_else(|| anyhow!("Failed to create PA context"))?;

    ctx.connect(None, FlagSet::NOFLAGS, None)?;

    logger::log_info("pulse-client", "Starting mainloop...");

    let introspector = Rc::new(RefCell::new(ctx.introspect()));
    let channel_cache = Rc::new(RefCell::new(HashMap::<u32, u8>::new()));
    let mut ready = false;
    let mut pending_cmds = VecDeque::new();

    loop {
        ml.iterate(false);

        while let Ok(cmd) = cmd_rx.try_recv() {
            if ready {
                handle_command(cmd, &introspector, &event_tx, &channel_cache);
            } else {
                pending_cmds.push_back(cmd);
            }
        }

        match ctx.get_state() {
            State::Ready if !ready => {
                ready = true;
                logger::log_info("pulse-client", "PulseAudio context is ready");

                setup_subscriptions(&mut ctx, &introspector, &event_tx, &channel_cache);
                request_initial_state(&introspector, &event_tx, &channel_cache);

                while let Some(cmd) = pending_cmds.pop_front() {
                    handle_command(cmd, &introspector, &event_tx, &channel_cache);
                }
            }
            State::Ready => {}
            State::Failed => bail!("PulseAudio context state = Failed"),
            State::Terminated => bail!("PulseAudio context state = Terminated"),
            State::Unconnected => bail!("PulseAudio context state = Unconnected"),
            State::Connecting | State::Authorizing | State::SettingName => {}
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

fn setup_subscriptions(
    ctx: &mut Context,
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    channel_cache: &ChannelCache,
) {
    let event_tx_sub = event_tx.clone();
    let introspector_sub = Rc::clone(introspector);
    let channel_cache_sub = Rc::clone(channel_cache);

    ctx.set_subscribe_callback(Some(Box::new(move |facility, operation, index| {
        let Some(facility) = facility else {
            logger::log_error(
                "pulse-client",
                "Received subscription event with unknown facility",
            );
            return;
        };

        let Some(operation) = operation else {
            logger::log_error(
                "pulse-client",
                "Received subscription event with unknown operation",
            );
            return;
        };

        match (facility, operation) {
            (Facility::SinkInput, Operation::New | Operation::Changed) => {
                request_app_volume_changed(index, &introspector_sub, &event_tx_sub, &channel_cache_sub);
            }
            (Facility::SinkInput, Operation::Removed) => {
                request_apps_list(&introspector_sub, &event_tx_sub);
            }
            (Facility::Sink, Operation::New | Operation::Changed) => {
                request_global_volume_changed(index, &introspector_sub, &event_tx_sub, &channel_cache_sub);
            }
            _ => {}
        }
    })));

    logger::log_debug(
        "pulse-client",
        "Subscribing to SINK and SINK_INPUT events...",
    );
    ctx.subscribe(
        InterestMaskSet::SINK | InterestMaskSet::SINK_INPUT,
        move |success| {
            if success {
                logger::log_info(
                    "pulse-client",
                    "Successfully subscribed to SINK and SINK_INPUT events",
                );
            } else {
                logger::log_error(
                    "pulse-client",
                    "Failed to subscribe to SINK and SINK_INPUT events",
                );
            }
        },
    );
}

fn request_initial_state(
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    channel_cache: &ChannelCache,
) {
    request_default_sink_volume(introspector, event_tx, true, channel_cache);
    request_app_volumes(introspector, event_tx, true, channel_cache);
}

fn handle_command(
    cmd: AudioCmd,
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    channel_cache: &ChannelCache,
) {
    match cmd {
        AudioCmd::AddOutput(_) | AudioCmd::ChangeOutput(_, _) => {}
        AudioCmd::SetGlobalVolume { sink_index, volume } => {
            set_global_volume(introspector, sink_index, volume, channel_cache, event_tx);
        }
        AudioCmd::ToggleGlobalMute { sink_index } => {
            toggle_global_mute(introspector, sink_index);
        }
        AudioCmd::SetAppVolume {
            sink_input_index,
            volume,
        } => {
            set_app_volume(introspector, sink_input_index, volume, channel_cache, event_tx);
        }
        AudioCmd::ToggleAppMute { sink_input_index } => {
            toggle_app_mute(introspector, sink_input_index);
        }
        AudioCmd::RequestGlobalVolume { sink_index } => {
            request_global_volume(introspector, event_tx, sink_index, channel_cache);
        }
        AudioCmd::RequestAppsList => {
            request_apps_list(introspector, event_tx);
        }
    }
}

fn request_global_volume(
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    sink_index: Option<u32>,
    channel_cache: &ChannelCache,
) {
    match sink_index {
        Some(index) => request_sink_volume_by_index(introspector, event_tx, index, true, channel_cache),
        None => request_default_sink_volume(introspector, event_tx, true, channel_cache),
    }
}

fn request_global_volume_changed(
    sink_index: u32,
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    channel_cache: &ChannelCache,
) {
    request_sink_volume_by_index(introspector, event_tx, sink_index, false, channel_cache);
}

fn request_sink_volume_by_index(
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    sink_index: u32,
    initial: bool,
    channel_cache: &ChannelCache,
) {
    let event_tx = event_tx.clone();
    let channel_cache_cb = Rc::clone(channel_cache);

    introspector
        .borrow()
        .get_sink_info_by_index(sink_index, move |res| match res {
            ListResult::Item(info) => {
                // Update channel cache
                channel_cache_cb.borrow_mut().insert(sink_index, info.volume.len() as u8);
                let output = OutputInfo::from_sink_info(&info.to_owned());
                send_global_volume_event(&event_tx, output, initial);
            }
            ListResult::End => {}
            ListResult::Error => {
                logger::log_error("pulse-client", "Error retrieving sink info");
            }
        });
}

fn request_default_sink_volume(
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    initial: bool,
    channel_cache: &ChannelCache,
) {
    let event_tx = event_tx.clone();
    let introspector_cb = Rc::clone(introspector);
    let channel_cache_cb = Rc::clone(channel_cache);

    introspector.borrow().get_server_info(move |info| {
        let Some(default_sink) = info.default_sink_name.as_deref() else {
            logger::log_error("pulse-client", "Default sink name is not available");
            return;
        };

        let event_tx = event_tx.clone();
        let channel_cache_cb2 = Rc::clone(&channel_cache_cb);

        introspector_cb
            .borrow()
            .get_sink_info_by_name(default_sink, move |res| match res {
                ListResult::Item(info) => {
                    // Update channel cache
                    channel_cache_cb2.borrow_mut().insert(info.index, info.volume.len() as u8);
                    let output = OutputInfo::from_sink_info(&info.to_owned());
                    send_global_volume_event(&event_tx, output, initial);
                }
                ListResult::End => {}
                ListResult::Error => {
                    logger::log_error("pulse-client", "Error retrieving default sink info");
                }
            });
    });
}

fn request_app_volumes(
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    initial: bool,
    channel_cache: &ChannelCache,
) {
    let event_tx = event_tx.clone();
    let channel_cache_cb = Rc::clone(channel_cache);

    introspector
        .borrow()
        .get_sink_input_info_list(move |res| match res {
            ListResult::Item(info) => {
                // Update channel cache
                channel_cache_cb.borrow_mut().insert(info.index, info.volume.len() as u8);
                let output = OutputInfo::from_sink_input_info(&info.to_owned());
                send_app_volume_event(&event_tx, output, initial);
            }
            ListResult::End => {}
            ListResult::Error => {
                logger::log_error("pulse-client", "Error retrieving sink input info list");
            }
        });
}

fn request_app_volume_changed(
    sink_input_index: u32,
    introspector: &SharedIntrospector,
    event_tx: &mpsc::Sender<AudioEvent>,
    channel_cache: &ChannelCache,
) {
    let event_tx = event_tx.clone();
    let channel_cache_cb = Rc::clone(channel_cache);

    introspector
        .borrow()
        .get_sink_input_info(sink_input_index, move |res| match res {
            ListResult::Item(info) => {
                // Update channel cache
                channel_cache_cb.borrow_mut().insert(sink_input_index, info.volume.len() as u8);
                let output = OutputInfo::from_sink_input_info(&info.to_owned());
                send_app_volume_event(&event_tx, output, false);
            }
            ListResult::End => {}
            ListResult::Error => {
                logger::log_error("pulse-client", "Error retrieving sink input info");
            }
        });
}

fn request_apps_list(introspector: &SharedIntrospector, event_tx: &mpsc::Sender<AudioEvent>) {
    let event_tx = event_tx.clone();
    let apps = Rc::new(RefCell::new(Vec::new()));
    let apps_cb = Rc::clone(&apps);

    introspector
        .borrow()
        .get_sink_input_info_list(move |res| match res {
            ListResult::Item(info) => {
                let output = OutputInfo::from_sink_input_info(&info.to_owned());
                apps_cb.borrow_mut().push(output);
            }
            ListResult::End => {
                let mut apps = apps_cb.borrow_mut();
                let list = std::mem::take(&mut *apps);
                let _ = event_tx.send(AudioEvent::AppsListUpdated { apps: list });
            }
            ListResult::Error => {
                logger::log_error("pulse-client", "Error retrieving sink input info list");
            }
        });
}

fn set_global_volume(
    introspector: &SharedIntrospector,
    sink_index: u32,
    volume: u32,
    channel_cache: &ChannelCache,
    event_tx: &mpsc::Sender<AudioEvent>,
) {
    let target = volume.min(100);
    let introspector_cb = Rc::clone(introspector);
    let event_tx_cb = event_tx.clone();

    // Try to use cached channel count first
    let channel_count = channel_cache.borrow().get(&sink_index).copied();

    if let Some(channels) = channel_count {
        // Use cached channel count - no need to query PulseAudio
        let Some(volumes) = build_channel_volumes(channels, target) else {
            logger::log_error("pulse-client", "Sink has no channels");
            return;
        };

        // Send optimistic event immediately
        let _ = event_tx_cb.send(AudioEvent::GlobalVolumeChanged {
            sink_index,
            volume: target,
            muted: false, // We don't know mute state, but it's optimistic anyway
        });

        let mut introspector = introspector_cb.borrow_mut();
        introspector.set_sink_volume_by_index(
            sink_index,
            &volumes,
            Some(Box::new(|success| {
                if !success {
                    logger::log_error("pulse-client", "Failed to set sink volume");
                }
            })),
        );
    } else {
        // Fallback: query PulseAudio if cache miss
        let channel_cache_cb = Rc::clone(channel_cache);
        introspector
            .borrow()
            .get_sink_info_by_index(sink_index, move |res| match res {
                ListResult::Item(info) => {
                    // Update cache
                    channel_cache_cb.borrow_mut().insert(sink_index, info.volume.len() as u8);
                    let Some(volumes) = build_channel_volumes(info.volume.len(), target) else {
                        logger::log_error("pulse-client", "Sink has no channels");
                        return;
                    };

                    // Send optimistic event immediately
                    let _ = event_tx_cb.send(AudioEvent::GlobalVolumeChanged {
                        sink_index,
                        volume: target,
                        muted: info.mute,
                    });

                    let mut introspector = introspector_cb.borrow_mut();
                    introspector.set_sink_volume_by_index(
                        sink_index,
                        &volumes,
                        Some(Box::new(|success| {
                            if !success {
                                logger::log_error("pulse-client", "Failed to set sink volume");
                            }
                        })),
                    );
                }
                ListResult::End => {}
                ListResult::Error => {
                    logger::log_error("pulse-client", "Error retrieving sink info for volume set");
                }
            });
    }
}

fn toggle_global_mute(introspector: &SharedIntrospector, sink_index: u32) {
    let introspector_cb = Rc::clone(introspector);

    introspector
        .borrow()
        .get_sink_info_by_index(sink_index, move |res| match res {
            ListResult::Item(info) => {
                let mute = !info.mute;
                let mut introspector = introspector_cb.borrow_mut();
                introspector.set_sink_mute_by_index(
                    sink_index,
                    mute,
                    Some(Box::new(|success| {
                        if !success {
                            logger::log_error("pulse-client", "Failed to set sink mute");
                        }
                    })),
                );
            }
            ListResult::End => {}
            ListResult::Error => {
                logger::log_error("pulse-client", "Error retrieving sink info for mute set");
            }
        });
}

fn set_app_volume(
    introspector: &SharedIntrospector,
    sink_input_index: u32,
    volume: u32,
    channel_cache: &ChannelCache,
    event_tx: &mpsc::Sender<AudioEvent>,
) {
    let target = volume.min(100);
    let introspector_cb = Rc::clone(introspector);
    let event_tx_cb = event_tx.clone();

    // Try to use cached channel count first
    let channel_count = channel_cache.borrow().get(&sink_input_index).copied();

    if let Some(channels) = channel_count {
        // Use cached channel count - no need to query PulseAudio
        let Some(volumes) = build_channel_volumes(channels, target) else {
            logger::log_error("pulse-client", "Sink input has no channels");
            return;
        };

        // Send optimistic event immediately (app name will be updated by subscription event)
        let _ = event_tx_cb.send(AudioEvent::AppVolumeChanged {
            sink_input_index,
            volume: target,
            muted: false, // We don't know mute state, but it's optimistic anyway
            app_name: String::new(), // Will be updated by subscription event
        });

        let mut introspector = introspector_cb.borrow_mut();
        introspector.set_sink_input_volume(
            sink_input_index,
            &volumes,
            Some(Box::new(|success| {
                if !success {
                    logger::log_error("pulse-client", "Failed to set sink input volume");
                }
            })),
        );
    } else {
        // Fallback: query PulseAudio if cache miss
        let channel_cache_cb = Rc::clone(channel_cache);
        introspector
            .borrow()
            .get_sink_input_info(sink_input_index, move |res| match res {
                ListResult::Item(info) => {
                    // Update cache
                    channel_cache_cb.borrow_mut().insert(sink_input_index, info.volume.len() as u8);
                    let Some(volumes) = build_channel_volumes(info.volume.len(), target) else {
                        logger::log_error("pulse-client", "Sink input has no channels");
                        return;
                    };

                    // Get app name from info
                    let app_name = match info.proplist.get_str(libpulse_binding::proplist::properties::APPLICATION_NAME) {
                        Some(name) => name.to_string(),
                        None => String::from("Unknown"),
                    };

                    // Send optimistic event immediately
                    let _ = event_tx_cb.send(AudioEvent::AppVolumeChanged {
                        sink_input_index,
                        volume: target,
                        muted: info.mute,
                        app_name,
                    });

                    let mut introspector = introspector_cb.borrow_mut();
                    introspector.set_sink_input_volume(
                        sink_input_index,
                        &volumes,
                        Some(Box::new(|success| {
                            if !success {
                                logger::log_error("pulse-client", "Failed to set sink input volume");
                            }
                        })),
                    );
                }
                ListResult::End => {}
                ListResult::Error => {
                    logger::log_error(
                        "pulse-client",
                        "Error retrieving sink input info for volume set",
                    );
                }
            });
    }
}

fn toggle_app_mute(introspector: &SharedIntrospector, sink_input_index: u32) {
    let introspector_cb = Rc::clone(introspector);

    introspector
        .borrow()
        .get_sink_input_info(sink_input_index, move |res| match res {
            ListResult::Item(info) => {
                let mute = !info.mute;
                let mut introspector = introspector_cb.borrow_mut();
                introspector.set_sink_input_mute(
                    sink_input_index,
                    mute,
                    Some(Box::new(|success| {
                        if !success {
                            logger::log_error("pulse-client", "Failed to set sink input mute");
                        }
                    })),
                );
            }
            ListResult::End => {}
            ListResult::Error => {
                logger::log_error(
                    "pulse-client",
                    "Error retrieving sink input info for mute set",
                );
            }
        });
}

fn build_channel_volumes(channels: u8, volume_percent: u32) -> Option<ChannelVolumes> {
    if channels == 0 {
        return None;
    }

    let mut volumes = ChannelVolumes::default();
    volumes.set(channels, percent_to_volume(volume_percent));
    Some(volumes)
}

fn percent_to_volume(percent: u32) -> Volume {
    let clamped = percent.min(100) as f64 / 100.0;
    Volume::from(VolumeLinear(clamped))
}

fn send_global_volume_event(
    event_tx: &mpsc::Sender<AudioEvent>,
    output: OutputInfo,
    initial: bool,
) {
    let event = if initial {
        AudioEvent::GlobalVolumeReceived {
            sink_index: output.index,
            volume: output.volume_level,
            muted: output.mute,
        }
    } else {
        AudioEvent::GlobalVolumeChanged {
            sink_index: output.index,
            volume: output.volume_level,
            muted: output.mute,
        }
    };

    let _ = event_tx.send(event);
}

fn send_app_volume_event(event_tx: &mpsc::Sender<AudioEvent>, output: OutputInfo, initial: bool) {
    let event = if initial {
        AudioEvent::AppVolumeReceived {
            sink_input_index: output.index,
            volume: output.volume_level,
            muted: output.mute,
            app_name: output.app_name,
        }
    } else {
        AudioEvent::AppVolumeChanged {
            sink_input_index: output.index,
            volume: output.volume_level,
            muted: output.mute,
            app_name: output.app_name,
        }
    };

    let _ = event_tx.send(event);
}
