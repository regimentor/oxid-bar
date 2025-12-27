use crate::backend::pulse::{client::Client, listen_pulse_backend::lesten_pulse_backend};

pub fn start_listening() -> anyhow::Result<()> {
    let mut client = Client::new();
    let cmd_tx = client.cmd_channels.tx.clone();
    std::thread::spawn(move || lesten_pulse_backend(cmd_tx));
    client.start_listening()?;

    Ok(())
}
