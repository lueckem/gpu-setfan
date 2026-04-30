use anyhow;

use crate::{fanspeed::FanSpeed, temperature::GPUTemperature};

pub trait GPUInterface {
    fn name(&self) -> &str;
    fn read_temperature(&self) -> anyhow::Result<GPUTemperature>;
    fn set_fan_speed(&mut self, target: FanSpeed) -> anyhow::Result<()>;
    fn restore_default_policy(&mut self) -> anyhow::Result<()>;
}
