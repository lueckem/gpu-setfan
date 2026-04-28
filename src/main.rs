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
const PROPORTIONAL_GAIN: f64 = 0.1;
const INTEGRAL_GAIN: f64 = 0.01;
const SMOOTHING_FACTOR: f64 = 0.5;

fn main() {
    logging::init_logging();

    info!("Program started");

    let nvml = Nvml::init().unwrap();
    let device = nvml.device_by_index(0).unwrap();
    let mut gpu = NvidiaGPU::init(device).unwrap();
    info!("Initialized Nvidia GPU '{}'", gpu.name);

    let mut fan_controller = FanController::new(
        TARGET_TEMPERATURE.try_into().unwrap(),
        FAN_ON_TEMPERATURE.try_into().unwrap(),
        FAN_OFF_TEMPERATURE.try_into().unwrap(),
        MIN_FAN_SPEED.try_into().unwrap(),
        PROPORTIONAL_GAIN,
        INTEGRAL_GAIN,
        SMOOTHING_FACTOR,
    );

    loop {
        let temp = gpu.read_temperature().unwrap();
        let target = fan_controller.eval(temp);
        gpu.set_fan_speed(target).unwrap();
        sleep(Duration::from_millis(UPDATE_PERIOD));
    }
}
