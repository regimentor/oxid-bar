use std::sync::mpsc;

use crate::backend::pulse::output_info::OutputInfo;

pub enum AudioCmd {
    AddOutput(OutputInfo),
    ChangeOutput(u32, OutputInfo),
}
pub enum AudioEvent {}

pub struct CmdChannels {
    pub tx: mpsc::Sender<AudioCmd>,
    pub rx: mpsc::Receiver<AudioCmd>,
}

pub struct EventChannels {
    pub tx: mpsc::Sender<AudioEvent>,
    pub rx: mpsc::Receiver<AudioEvent>,
}

pub struct Client {
    pub cmd_channels: CmdChannels,
    pub event_channels: EventChannels,
    outputs: Vec<OutputInfo>,
}

impl Client {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<AudioCmd>();
        let (event_tx, event_rx) = mpsc::channel::<AudioEvent>();

        Client {
            outputs: vec![],
            cmd_channels: CmdChannels {
                tx: cmd_tx,
                rx: cmd_rx,
            },
            event_channels: EventChannels {
                tx: event_tx,
                rx: event_rx,
            },
        }
    }

    pub fn start_listening(&mut self) -> anyhow::Result<()> {
        while let Ok(msg) = self.cmd_channels.rx.recv() {
            match msg {
                AudioCmd::AddOutput(si) => {
                    logger::log_debug("pulse-client", format!("Adding output: {}", si));
                    self.outputs.push(si)
                }
                AudioCmd::ChangeOutput(index, info) => {
                    logger::log_debug(
                        "pulse-client",
                        format!("Changing output: index={}, output={}", index, info),
                    );
                }
            }
        }

        Ok(())
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
