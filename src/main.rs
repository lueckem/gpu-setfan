use std::{thread::sleep, time::Duration};

use nvml_wrapper::Nvml;
use tracing::info;

use crate::{interface::GPUInterface, nvidia::NvidiaGPU};

mod fanspeed;
mod interface;
mod logging;
mod nvidia;
mod temperature;

fn main() {
    logging::init_logging();

    info!("Program started");

    let nvml = Nvml::init().unwrap();
    let device = nvml.device_by_index(0).unwrap();
    let gpu = NvidiaGPU::init(device).unwrap();
    info!("Initialized Nvidia GPU '{}'", gpu.name);

    loop {
        let temp = gpu.read_temperature().unwrap();
        let temp: f64 = temp.into();
        println!("{temp}");

        sleep(Duration::from_millis(1000));
    }
}
