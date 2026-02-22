use crate::backend::pulse::output_info::OutputInfo;

pub enum AudioCmd {
    AddOutput(OutputInfo),
    ChangeOutput(u32, OutputInfo),
}
pub enum AudioEvent {}
