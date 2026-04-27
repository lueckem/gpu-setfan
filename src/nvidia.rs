use anyhow::{self, Context};
use nvml_wrapper::{Device, enum_wrappers::device::TemperatureSensor};
use tracing::debug;

use crate::{fanspeed::FanSpeed, interface::GPUInterface, temperature::GPUTemperature};

pub struct NvidiaGPU<'a> {
    device: Device<'a>,
}

// TODO: store num_fans?

impl<'a> NvidiaGPU<'a> {
    pub fn new(device: Device<'a>) -> Self {
        Self { device }
    }
}

impl GPUInterface for NvidiaGPU<'_> {
    fn read_temperature(&self) -> anyhow::Result<GPUTemperature> {
        let temp = self
            .device
            .temperature(TemperatureSensor::Gpu)
            .context("Failed to read GPU temperature")?;
        debug!("Read temperature {temp}");
        let temp: GPUTemperature = (temp as f64).try_into()?;
        Ok(temp)
    }

    fn set_fan_speed(&mut self, target: FanSpeed) -> anyhow::Result<()> {
        let target: u32 = target.into();
        let fan_count = self
            .device
            .num_fans()
            .context("Failed to get number of fans")?;
        debug!("Number of fans: {fan_count}");
        for fan_index in 0..fan_count {
            self.device
                .set_fan_speed(fan_index, target)
                .context("Failed to set fan speed")?;
        }
        debug!("Fan speed set to {}", target);
        Ok(())
    }

    fn restore_default_policy(&mut self) -> anyhow::Result<()> {
        let fan_count = self
            .device
            .num_fans()
            .context("Failed to get number of fans")?;
        for fan_index in 0..fan_count {
            self.device.set_default_fan_speed(fan_index)?;
        }
        debug!("Restored default fan policy");
        Ok(())
    }
}
