use anyhow;

use crate::{fanspeed::FanSpeed, temperature::GPUTemperature};

pub trait GPUInterface {
    fn name(&self) -> &str;
    fn read_temperature(&self) -> anyhow::Result<GPUTemperature>;
    fn set_fan_speed(&mut self, target: FanSpeed) -> anyhow::Result<()>;
    fn restore_default_policy(&mut self) -> anyhow::Result<()>;
}

/// Construct a String listing the names of the GPUs separated by comma
pub fn gpus_to_string(gpus: &[Box<dyn GPUInterface + '_>]) -> String {
    let mut s = String::new();

    if gpus.is_empty() {
        return s;
    }

    s.push('\'');
    s.push_str(gpus[0].name());
    s.push('\'');

    for i in 1..gpus.len() {
        s.push_str(", ");
        s.push('\'');
        s.push_str(gpus[i].name());
        s.push('\'');
    }
    s
}
