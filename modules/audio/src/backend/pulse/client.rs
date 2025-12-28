use crate::backend::pulse::output_info::OutputInfo;

pub enum AudioCmd {
    AddOutput(OutputInfo),
    ChangeOutput(u32, OutputInfo),
    SetGlobalVolume { sink_index: u32, volume: u32 },
    ToggleGlobalMute { sink_index: u32 },
    SetAppVolume { sink_input_index: u32, volume: u32 },
    ToggleAppMute { sink_input_index: u32 },
    RequestGlobalVolume { sink_index: Option<u32> },
    RequestAppsList,
}

pub enum AudioEvent {
    GlobalVolumeChanged {
        sink_index: u32,
        volume: u32,
        muted: bool,
    },
    AppVolumeChanged {
        sink_input_index: u32,
        volume: u32,
        muted: bool,
        app_name: String,
    },
    AppsListUpdated {
        apps: Vec<OutputInfo>,
    },
    GlobalVolumeReceived {
        sink_index: u32,
        volume: u32,
        muted: bool,
    },
    AppVolumeReceived {
        sink_input_index: u32,
        volume: u32,
        muted: bool,
        app_name: String,
    },
}
