use anyhow;

use crate::{fanspeed::FanSpeed, temperature::GPUTemperature};

pub trait GPUInterface {
    fn read_temperature(&self) -> anyhow::Result<GPUTemperature>;
    fn set_fan_speed(&self, target: FanSpeed) -> anyhow::Result<()>;
    fn restore_default_policy(&self) -> anyhow::Result<()>;
}
