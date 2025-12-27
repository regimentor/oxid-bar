use std::sync::{Arc, mpsc};

use anyhow::{anyhow, bail};
use libpulse_binding::{
    callbacks::ListResult,
    context::{
        Context, FlagSet, State,
        subscribe::{Facility, InterestMaskSet, Operation},
    },
    mainloop::standard::Mainloop,
};

use crate::backend::pulse::{client::AudioCmd, output_info::OutputInfo};
pub fn lesten_pulse_backend(cmd_tx: mpsc::Sender<AudioCmd>) -> anyhow::Result<()> {
    let mut ml = Mainloop::new().ok_or_else(|| anyhow!("Failed to create PA mainloop"))?;

    let mut ctx = Context::new(&ml, "oxid-bar-audio")
        .ok_or_else(|| anyhow!("Failed to create PA context"))?;

    ctx.connect(None, FlagSet::NOFLAGS, None)?;

    logger::log_info("pulse-client", "Starting mainloop...");

    let mut requested = false;
    loop {
        ml.iterate(true);
        match ctx.get_state() {
            State::Ready if !requested => {
                requested = true;
                logger::log_info("pulse-client", "PulseAudio context is ready");

                let introspector = ctx.introspect();
                let introspector = Arc::new(introspector);
                let cmd_tx_sync_clone = cmd_tx.clone();
                introspector.get_sink_input_info_list(move |res| match res {
                    ListResult::Item(info) => {
                        logger::log_debug(
                            "pulse-client",
                            format!("Retrieved sink input info: index={}", info.index),
                        );
                        let output_info = OutputInfo::from_sink_input_info(&info.to_owned());
                        let _ = cmd_tx_sync_clone.send(AudioCmd::AddOutput(output_info));
                    }
                    ListResult::End => {
                        logger::log_debug(
                            "pulse-client",
                            "Finished retrieving sink input info list",
                        );
                    }
                    ListResult::Error => {
                        logger::log_error("pulse-client", "Error retrieving sink input info list");
                    }
                });

                let cmd_tx_sub_clone = cmd_tx.clone();
                let introspector_clone = introspector.clone();
                ctx.set_subscribe_callback(Some(Box::new({
                    move |facility, operation, index| {
                        logger::log_debug(
                            "pulse-client",
                            format!(
                                "Received subscription event: facility={:?}, operation={:?}, index={}",
                                facility, operation, index
                            ),
                        );
                        let facility = if let Some(facility) = facility {
                            facility
                        } else {
                            logger::log_error(
                                "pulse-client",
                                "Received subscription event with unknown facility",
                            );
                            return;
                        };

                        let operation = if let Some(operation) = operation {
                            operation
                        } else {
                            logger::log_error(
                                "pulse-client",
                                "Received subscription event with unknown operation",
                            );
                            return;
                        };

                        match (facility, operation) {
                            (Facility::SinkInput, Operation::New | Operation::Changed) => {
                                let cmd_tx_inner = cmd_tx_sub_clone.clone();
                                introspector_clone.get_sink_input_info(index, move |res| {
                                    match res {
                                        ListResult::Item(info) => {
                                            logger::log_debug(
                                                "pulse-client",
                                                format!(
                                                    "Retrieved sink input info for changed/new event: index={}",
                                                    info.index
                                                ),
                                            );
                                            let output_info =
                                                OutputInfo::from_sink_input_info(&info.to_owned());
                                            let _ = cmd_tx_inner
                                                .send(AudioCmd::ChangeOutput(index, output_info));
                                        }
                                        ListResult::End => {
                                            logger::log_debug(
                                                "pulse-client",
                                                "Finished retrieving sink input info for changed/new event",
                                            );
                                        }
                                        ListResult::Error => {
                                            logger::log_error(
                                                "pulse-client",
                                                "Error retrieving sink input info for changed/new event",
                                            );
                                        }
                                    }
                                });
                            }
                            _ => {
                                logger::log_debug(
                                    "pulse-client",
                                    "Ignoring non-sink-input new/changed event",
                                );
                            }
                        }
                    }
                })));

                logger::log_debug("pulse-client", "Subscribing to SINK_INPUT events...");
                ctx.subscribe(InterestMaskSet::SINK_INPUT, move |success| {
                    if success {
                        logger::log_info(
                            "pulse-client",
                            "Successfully subscribed to SINK_INPUT events",
                        );
                    } else {
                        logger::log_error(
                            "pulse-client",
                            "Failed to subscribe to SINK_INPUT events",
                        );
                    }
                });
            }
            State::Ready => {
                // Already requested, do nothing
            }
            State::Failed => bail!("PulseAudio context state = Failed"),
            State::Terminated => bail!("PulseAudio context state = Terminated"),
            State::Unconnected => bail!("PulseAudio context state = Unconnected"),
            State::Connecting => {
                // Still connecting, do nothing
            }
            State::Authorizing => {
                // Still authorizing, do nothing
            }
            State::SettingName => {
                // Still setting name, do nothing
            }
        }
    }
}
