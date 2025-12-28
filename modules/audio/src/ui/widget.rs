use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use glib::{ControlFlow, timeout_add_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, GestureClick, Label, PositionType};

use crate::backend::pulse::client::{AudioCmd, AudioEvent};
use crate::backend::pulse::listen_pulse_backend::lesten_pulse_backend;

use super::popup::AudioPopup;

pub struct AudioWidget {
    icon_container: GtkBox,
    icon_label: Label,
    current_global_volume: u32,
    is_global_muted: bool,
    expected_global_volume: RefCell<Option<u32>>,
}

impl AudioWidget {
    pub fn new(icon_container: GtkBox) -> Self {
        let icon_label = Label::new(None);
        icon_label.add_css_class("audio-widget-icon");
        icon_container.append(&icon_label);

        let widget = Self {
            icon_container,
            icon_label,
            current_global_volume: 0,
            is_global_muted: false,
            expected_global_volume: RefCell::new(None),
        };

        widget.update_icon();
        widget
    }

    pub fn handle_event(&mut self, event: &AudioEvent) {
        match event {
            AudioEvent::GlobalVolumeChanged { volume, muted, .. } => {
                // Check if this event matches our expected value
                let should_ignore = {
                    let expected = self.expected_global_volume.borrow();
                    expected.map_or(false, |expected_volume| expected_volume == *volume)
                };
                
                if should_ignore {
                    // This is the event we expected from our own change
                    // Clear expected value and only update mute state if needed
                    *self.expected_global_volume.borrow_mut() = None;
                    // Update mute state even if volume matches
                    if self.is_global_muted != *muted {
                        self.is_global_muted = *muted;
                        self.update_icon();
                    }
                    return;
                }
                
                // This is an external change or unexpected value - update UI
                self.current_global_volume = *volume;
                self.is_global_muted = *muted;
                // Clear expected value since we've processed it
                *self.expected_global_volume.borrow_mut() = None;
                self.update_icon();
            }
            AudioEvent::GlobalVolumeReceived { volume, muted, .. } => {
                // Initial state - always update and clear expected value
                *self.expected_global_volume.borrow_mut() = None;
                self.current_global_volume = *volume;
                self.is_global_muted = *muted;
                self.update_icon();
            }
            _ => {}
        }
    }

    fn update_icon(&self) {
        let text = if self.is_global_muted {
            "MUTE".to_string()
        } else {
            format!("VOL {}%", self.current_global_volume)
        };

        self.icon_label.set_label(&text);
    }
}

pub fn build_ui(icon_container: &GtkBox, root: &GtkBox) -> Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<AudioCmd>();
    let (event_tx, event_rx) = mpsc::channel::<AudioEvent>();

    let widget = Rc::new(RefCell::new(AudioWidget::new(icon_container.clone())));

    let popup = Rc::new(RefCell::new(AudioPopup::new(root, cmd_tx.clone())));
    popup.borrow().popup_window().set_parent(icon_container);
    popup
        .borrow()
        .popup_window()
        .set_position(PositionType::Bottom);

    let click = GestureClick::new();
    let popup_for_click = popup.clone();
    click.connect_pressed(move |_gesture, _, _, _| {
        let popup_ref = popup_for_click.borrow();
        if popup_ref.popup_window().is_visible() {
            popup_ref.popup_window().popdown();
        } else {
            popup_ref.popup_window().popup();
        }
    });
    icon_container.add_controller(click);

    std::thread::spawn(move || {
        if let Err(error) = lesten_pulse_backend(cmd_rx, event_tx) {
            logger::log_error("AudioWidget::build_ui", error);
        }
    });

    let widget_for_events = widget.clone();
    let popup_for_events = popup.clone();
    timeout_add_local(Duration::from_millis(50), move || {
        loop {
            match event_rx.try_recv() {
                Ok(event) => {
                    if let Ok(mut widget) = widget_for_events.try_borrow_mut() {
                        widget.handle_event(&event);
                    }
                    if let Ok(mut popup) = popup_for_events.try_borrow_mut() {
                        popup.handle_event(event);
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return ControlFlow::Break,
            }
        }

        ControlFlow::Continue
    });

    send_cmd(&cmd_tx, AudioCmd::RequestGlobalVolume { sink_index: None });
    send_cmd(&cmd_tx, AudioCmd::RequestAppsList);

    Ok(())
}

fn send_cmd(cmd_tx: &mpsc::Sender<AudioCmd>, cmd: AudioCmd) {
    if let Err(error) = cmd_tx.send(cmd) {
        logger::log_error("AudioWidget::send_cmd", error);
    }
}
