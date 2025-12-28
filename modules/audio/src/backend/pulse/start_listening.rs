use std::sync::mpsc;

use crate::backend::pulse::client::{AudioCmd, AudioEvent};
use crate::backend::pulse::listen_pulse_backend::lesten_pulse_backend;

pub fn start_listening() -> anyhow::Result<()> {
    let (_cmd_tx, cmd_rx) = mpsc::channel::<AudioCmd>();
    let (event_tx, _event_rx) = mpsc::channel::<AudioEvent>();

    lesten_pulse_backend(cmd_rx, event_tx)
}
