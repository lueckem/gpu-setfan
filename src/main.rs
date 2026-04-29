use std::{thread::sleep, time::Duration};

use nvml_wrapper::Nvml;
use tracing::info;

use crate::{fan_controller::FanController, interface::GPUInterface, nvidia::NvidiaGPU};

mod fan_controller;
mod fanspeed;
mod interface;
mod logging;
mod nvidia;
mod pi_controller;
mod temperature;

const UPDATE_PERIOD: u64 = 1000; // in ms
const TARGET_TEMPERATURE: f64 = 80.0;
const FAN_ON_TEMPERATURE: f64 = 65.0;
const FAN_OFF_TEMPERATURE: f64 = 55.0;
const MIN_FAN_SPEED: u32 = 30;

fn main() -> anyhow::Result<()> {
    logging::init_logging();
    info!("Program started");

    // detect and initialize gpus
    let nvml_res = Nvml::init();
    let mut gpus: Vec<Box<dyn GPUInterface>> = Vec::new();
    if let Ok(ref nvml) = nvml_res {
        if let Ok(gpus_nvidia) = initialize_nvidia(nvml) {
            gpus.extend(gpus_nvidia);
        }
    }

    if gpus.is_empty() {
        anyhow::bail!("Could not detect any GPU");
    }

    // TODO: create one controller per GPU
    let mut fan_controller = FanController::new(
        TARGET_TEMPERATURE.try_into().unwrap(),
        FAN_ON_TEMPERATURE.try_into().unwrap(),
        FAN_OFF_TEMPERATURE.try_into().unwrap(),
        MIN_FAN_SPEED.try_into().unwrap(),
    );

    // TODO: go over each (gpu, controller) pair
    loop {
        for gpu in &mut gpus {
            let temp = gpu.read_temperature().unwrap();
            let target = fan_controller.eval(temp);
            gpu.set_fan_speed(target).unwrap();
        }

        sleep(Duration::from_millis(UPDATE_PERIOD));
    }
}

fn initialize_nvidia(nvml: &Nvml) -> anyhow::Result<Vec<Box<dyn GPUInterface + '_>>> {
    let num_devices = nvml.device_count()?;
    let mut gpus: Vec<Box<dyn GPUInterface>> = Vec::with_capacity(num_devices as usize);
    for i in 0..num_devices {
        let device = nvml.device_by_index(i)?;
        let gpu = NvidiaGPU::init(device)?;
        info!("Initialized Nvidia GPU '{}'", gpu.name);
        gpus.push(Box::new(gpu));
    }
    Ok(gpus)
}
