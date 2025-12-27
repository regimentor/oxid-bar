use libpulse_binding::{
    context::introspect::{SinkInfo, SinkInputInfo},
    proplist::properties::APPLICATION_NAME,
    volume::Volume,
};

pub struct OutputInfo {
    pub index: u32,
    pub sink: u32,
    pub client: Option<u32>,
    pub mute: bool,
    pub name: String,
    pub app_name: String,
    pub volume_level: u32,
}

impl std::fmt::Display for OutputInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OutputInfo {{ index: {}, sink: {}, client: {:?}, mute: {}, name: {}, app_name: {}, volume_level: {} }}",
            self.index,
            self.sink,
            self.client,
            self.mute,
            self.name,
            self.app_name,
            self.volume_level
        )
    }
}

impl OutputInfo {
    pub fn from_sink_input_info(input: &SinkInputInfo<'static>) -> OutputInfo {
        let name = match &input.name {
            Some(n) => n.to_string(),
            None => String::from("Unknown"),
        };

        let app_name = match input.proplist.get_str(APPLICATION_NAME) {
            Some(n) => n,
            None => String::from("Unknown"),
        };

        let volume_level = (input.volume.avg().0 as f64 / Volume::NORMAL.0 as f64 * 100.0) as u32;

        OutputInfo {
            index: input.index,
            sink: input.sink,
            client: input.client,
            mute: input.mute,
            name,
            app_name,
            volume_level,
        }
    }

    pub fn from_sink_info(sink: &SinkInfo<'static>) -> OutputInfo {
        let name = match &sink.name {
            Some(n) => n.to_string(),
            None => String::from("Unknown"),
        };

        let app_name = match sink.proplist.get_str(APPLICATION_NAME) {
            Some(n) => n,
            None => String::from("Unknown"),
        };

        let volume_level = (sink.volume.avg().0 as f64 / Volume::NORMAL.0 as f64 * 100.0) as u32;

        OutputInfo {
            index: sink.index,
            sink: sink.index,
            client: None,
            mute: sink.mute,
            name,
            app_name,
            volume_level,
        }
    }
}
