use hyprland::event_listener::EventListener;
use std::{sync::mpsc, thread};

/// Запускает слушатель событий Hyprland в отдельном потоке
pub fn start_hyprland_event_listener(tx: mpsc::Sender<()>) {
    thread::spawn(move || {
        let mut listener = EventListener::new();

        let send = |desc: &str, tx: &mpsc::Sender<()>| {
            if let Err(err) = tx.send(()) {
                logger::log_error(&format!("HyprlandListener({})", desc), err);
            }
        };

        let tx_ws = tx.clone();
        listener.add_workspace_changed_handler(move |_| send("workspace_changed", &tx_ws));

        let tx_added = tx.clone();
        listener.add_workspace_added_handler(move |_| send("workspace_added", &tx_added));

        let tx_deleted = tx.clone();
        listener.add_workspace_deleted_handler(move |_| send("workspace_deleted", &tx_deleted));

        let tx_moved = tx.clone();
        listener.add_workspace_moved_handler(move |_| send("workspace_moved", &tx_moved));

        let tx_renamed = tx.clone();
        listener.add_workspace_renamed_handler(move |_| send("workspace_renamed", &tx_renamed));

        let tx_open = tx.clone();
        listener.add_window_opened_handler(move |_| send("window_opened", &tx_open));

        let tx_close = tx.clone();
        listener.add_window_closed_handler(move |_| send("window_closed", &tx_close));

        if let Err(err) = listener.start_listener() {
            logger::log_error("HyprlandListener::start", err);
        }
    });
}

