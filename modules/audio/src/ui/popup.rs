use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;

use gtk4::glib::SignalHandlerId;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, Popover, Scale};

use crate::backend::pulse::client::{AudioCmd, AudioEvent};
use crate::backend::pulse::output_info::OutputInfo;

pub struct AudioPopup {
    popup_window: Popover,
    cmd_tx: mpsc::Sender<AudioCmd>,
    global_volume_scale: Scale,
    global_volume_label: Label,
    global_volume_handler_id: RefCell<Option<SignalHandlerId>>,
    global_mute_button: Button,
    apps_list: GtkBox,
    apps: Vec<AppControl>,
    global_sink_index: Rc<RefCell<Option<u32>>>,
    expected_global_volume: Rc<RefCell<Option<u32>>>,
    expected_app_volumes: Rc<RefCell<HashMap<u32, u32>>>,
}

struct AppControl {
    app_name: String,
    sink_input_index: u32,
    volume_scale: Scale,
    volume_handler_id: RefCell<Option<SignalHandlerId>>,
    mute_button: Button,
    volume_label: Label,
}

impl AudioPopup {
    pub fn new(_root: &GtkBox, cmd_tx: mpsc::Sender<AudioCmd>) -> Self {
        let popup_window = Popover::new();
        popup_window.add_css_class("audio-popup");
        popup_window.set_has_arrow(false);
        popup_window.set_autohide(true);
        let content = GtkBox::new(Orientation::Vertical, 8);
        content.add_css_class("audio-popup");

        let global_row = GtkBox::new(Orientation::Horizontal, 8);
        let global_label = Label::new(Some("Volume"));
        global_label.set_xalign(0.0);

        let global_volume_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        global_volume_scale.set_hexpand(true);

        let global_volume_label = Label::new(Some("0%"));
        let global_mute_button = Button::with_label("Mute");

        global_row.append(&global_label);
        global_row.append(&global_volume_scale);
        global_row.append(&global_volume_label);
        global_row.append(&global_mute_button);

        let apps_list = GtkBox::new(Orientation::Vertical, 6);
        apps_list.set_margin_top(6);

        content.append(&global_row);
        content.append(&apps_list);
        popup_window.set_child(Some(&content));

        let global_sink_index = Rc::new(RefCell::new(None));
        let expected_global_volume = Rc::new(RefCell::new(None));

        let cmd_tx_for_scale = cmd_tx.clone();
        let sink_for_scale = global_sink_index.clone();
        let expected_global_volume_for_scale = expected_global_volume.clone();
        let global_volume_label_clone = global_volume_label.clone();
        let handler_id = global_volume_scale.connect_value_changed(move |scale| {
            let Some(sink_index) = *sink_for_scale.borrow() else {
                return;
            };
            let value = scale.value().round().clamp(0.0, 100.0) as u32;
            // Store expected value to ignore matching events from PulseAudio
            *expected_global_volume_for_scale.borrow_mut() = Some(value);
            // Optimistic UI update: update label immediately
            global_volume_label_clone.set_label(&format!("{}%", value));
            send_cmd(
                &cmd_tx_for_scale,
                AudioCmd::SetGlobalVolume {
                    sink_index,
                    volume: value,
                },
            );
        });

        let cmd_tx_for_mute = cmd_tx.clone();
        let sink_for_mute = global_sink_index.clone();
        global_mute_button.connect_clicked(move |_| {
            let Some(sink_index) = *sink_for_mute.borrow() else {
                return;
            };
            send_cmd(&cmd_tx_for_mute, AudioCmd::ToggleGlobalMute { sink_index });
        });

        Self {
            popup_window,
            cmd_tx,
            global_volume_scale,
            global_volume_label,
            global_volume_handler_id: RefCell::new(Some(handler_id)),
            global_mute_button,
            apps_list,
            apps: Vec::new(),
            global_sink_index,
            expected_global_volume,
            expected_app_volumes: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn popup_window(&self) -> &Popover {
        &self.popup_window
    }

    pub fn update_global_volume(&self, volume: u32, muted: bool) {
        let value = (volume.min(100)) as f64;
        if (self.global_volume_scale.value() - value).abs() > f64::EPSILON {
            // Block signal to prevent feedback loop when updating programmatically
            if let Some(handler_id) = self.global_volume_handler_id.borrow().as_ref() {
                self.global_volume_scale.block_signal(handler_id);
            }
            self.global_volume_scale.set_value(value);
            if let Some(handler_id) = self.global_volume_handler_id.borrow().as_ref() {
                self.global_volume_scale.unblock_signal(handler_id);
            }
        }
        self.global_volume_label
            .set_label(&format!("{}%", volume.min(100)));
        self.global_mute_button
            .set_label(if muted { "Unmute" } else { "Mute" });
    }

    pub fn update_apps_list(&mut self, apps: Vec<OutputInfo>) {
        self.apps.clear();
        while let Some(child) = self.apps_list.first_child() {
            self.apps_list.remove(&child);
        }

        let expected_app_volumes = self.expected_app_volumes.clone();

        for app in apps {
            let row = GtkBox::new(Orientation::Horizontal, 8);

            let name_label = Label::new(Some(&app.app_name));
            name_label.set_xalign(0.0);
            name_label.set_hexpand(true);

            let volume_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
            volume_scale.set_value(app.volume_level as f64);
            volume_scale.set_hexpand(true);

            let volume_label = Label::new(Some(&format!("{}%", app.volume_level.min(100))));
            let mute_button = Button::with_label(if app.mute { "Unmute" } else { "Mute" });

            row.append(&name_label);
            row.append(&volume_scale);
            row.append(&volume_label);
            row.append(&mute_button);

            let cmd_tx_for_scale = self.cmd_tx.clone();
            let volume_label_clone = volume_label.clone();
            let sink_input_index = app.index;
            let expected_app_volumes_for_scale = expected_app_volumes.clone();
            let app_handler_id = volume_scale.connect_value_changed(move |scale| {
                let value = scale.value().round().clamp(0.0, 100.0) as u32;
                // Store expected value to ignore matching events from PulseAudio
                expected_app_volumes_for_scale.borrow_mut().insert(sink_input_index, value);
                // Optimistic UI update: update label immediately
                volume_label_clone.set_label(&format!("{}%", value));
                send_cmd(
                    &cmd_tx_for_scale,
                    AudioCmd::SetAppVolume {
                        sink_input_index,
                        volume: value,
                    },
                );
            });

            let cmd_tx_for_mute = self.cmd_tx.clone();
            let sink_input_index = app.index;
            mute_button.connect_clicked(move |_| {
                send_cmd(
                    &cmd_tx_for_mute,
                    AudioCmd::ToggleAppMute { sink_input_index },
                );
            });

            self.apps_list.append(&row);
            self.apps.push(AppControl {
                app_name: app.app_name,
                sink_input_index: app.index,
                volume_scale,
                volume_handler_id: RefCell::new(Some(app_handler_id)),
                mute_button,
                volume_label,
            });
        }
    }

    pub fn handle_event(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::GlobalVolumeChanged {
                sink_index,
                volume,
                muted,
            } => {
                *self.global_sink_index.borrow_mut() = Some(sink_index);
                // Check if this event matches our expected value
                let should_ignore = {
                    let expected = self.expected_global_volume.borrow();
                    expected.map_or(false, |expected_volume| expected_volume == volume)
                };
                
                if should_ignore {
                    // This is the event we expected from our own change
                    // Clear expected value and only update mute state if needed
                    *self.expected_global_volume.borrow_mut() = None;
                    // Update mute state even if volume matches
                    let current_muted = self
                        .global_mute_button
                        .label()
                        .as_deref()
                        .map(|s| s == "Unmute")
                        .unwrap_or(false);
                    if current_muted != muted {
                        self.global_mute_button
                            .set_label(if muted { "Unmute" } else { "Mute" });
                    }
                    return;
                }
                
                // This is an external change or unexpected value - update UI
                self.update_global_volume(volume, muted);
                // Clear expected value since we've processed it
                *self.expected_global_volume.borrow_mut() = None;
            }
            AudioEvent::GlobalVolumeReceived {
                sink_index,
                volume,
                muted,
            } => {
                *self.global_sink_index.borrow_mut() = Some(sink_index);
                // Initial state - always update and clear expected value
                *self.expected_global_volume.borrow_mut() = None;
                self.update_global_volume(volume, muted);
            }
            AudioEvent::AppsListUpdated { apps } => {
                self.update_apps_list(apps);
            }
            AudioEvent::AppVolumeChanged {
                sink_input_index,
                volume,
                muted,
                ..
            } => {
                // Check if this event matches our expected value
                let should_ignore = {
                    let expected = self.expected_app_volumes.borrow();
                    expected
                        .get(&sink_input_index)
                        .map_or(false, |expected_volume| *expected_volume == volume)
                };
                
                if should_ignore {
                    // This is the event we expected from our own change
                    // Clear expected value and only update mute state if needed
                    self.expected_app_volumes.borrow_mut().remove(&sink_input_index);
                    // Update mute state even if volume matches
                    if let Some(app) = self
                        .apps
                        .iter()
                        .find(|app| app.sink_input_index == sink_input_index)
                    {
                        let current_muted = app
                            .mute_button
                            .label()
                            .as_deref()
                            .map(|s| s == "Unmute")
                            .unwrap_or(false);
                        if current_muted != muted {
                            app.mute_button
                                .set_label(if muted { "Unmute" } else { "Mute" });
                        }
                    }
                    return;
                }
                
                // This is an external change or unexpected value - update UI
                self.update_app_volume(sink_input_index, volume, muted);
                // Clear expected value since we've processed it
                self.expected_app_volumes.borrow_mut().remove(&sink_input_index);
            }
            AudioEvent::AppVolumeReceived {
                sink_input_index,
                volume,
                muted,
                ..
            } => {
                // Initial state - always update and clear expected value
                self.expected_app_volumes.borrow_mut().remove(&sink_input_index);
                self.update_app_volume(sink_input_index, volume, muted);
            }
        }
    }

    pub fn on_global_volume_changed(&self, value: f64) {
        let Some(sink_index) = *self.global_sink_index.borrow() else {
            return;
        };
        let volume = value.round().clamp(0.0, 100.0) as u32;
        // Store expected value to ignore matching events from PulseAudio
        *self.expected_global_volume.borrow_mut() = Some(volume);
        send_cmd(
            &self.cmd_tx,
            AudioCmd::SetGlobalVolume { sink_index, volume },
        );
    }

    pub fn on_global_mute_clicked(&self) {
        let Some(sink_index) = *self.global_sink_index.borrow() else {
            return;
        };
        send_cmd(&self.cmd_tx, AudioCmd::ToggleGlobalMute { sink_index });
    }

    pub fn on_app_volume_changed(&self, sink_input_index: u32, value: f64) {
        let volume = value.round().clamp(0.0, 100.0) as u32;
        // Store expected value to ignore matching events from PulseAudio
        self.expected_app_volumes
            .borrow_mut()
            .insert(sink_input_index, volume);
        send_cmd(
            &self.cmd_tx,
            AudioCmd::SetAppVolume {
                sink_input_index,
                volume,
            },
        );
    }

    pub fn on_app_mute_clicked(&self, sink_input_index: u32) {
        send_cmd(&self.cmd_tx, AudioCmd::ToggleAppMute { sink_input_index });
    }

    fn update_app_volume(&self, sink_input_index: u32, volume: u32, muted: bool) {
        let Some(app) = self
            .apps
            .iter()
            .find(|app| app.sink_input_index == sink_input_index)
        else {
            return;
        };

        let value = volume.min(100) as f64;
        if (app.volume_scale.value() - value).abs() > f64::EPSILON {
            // Block signal to prevent feedback loop when updating programmatically
            if let Some(handler_id) = app.volume_handler_id.borrow().as_ref() {
                app.volume_scale.block_signal(handler_id);
            }
            app.volume_scale.set_value(value);
            if let Some(handler_id) = app.volume_handler_id.borrow().as_ref() {
                app.volume_scale.unblock_signal(handler_id);
            }
        }
        app.volume_label.set_label(&format!("{}%", volume.min(100)));
        app.mute_button
            .set_label(if muted { "Unmute" } else { "Mute" });
    }
}

fn send_cmd(cmd_tx: &mpsc::Sender<AudioCmd>, cmd: AudioCmd) {
    if let Err(error) = cmd_tx.send(cmd) {
        logger::log_error("AudioPopup::send_cmd", error);
    }
}
