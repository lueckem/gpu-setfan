use std::{thread::sleep, time::Duration};

use nvml_wrapper::Nvml;
use tracing::{debug, error, info, warn};

use crate::{
    fan_controller::FanController,
    interface::{GPUInterface, gpus_to_string},
    nvidia::initialize_nvidia_gpus,
};

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
        match initialize_nvidia_gpus(nvml) {
            Ok(gpus_nvidia) => gpus.extend(gpus_nvidia),
            Err(err) => warn!(
                "Nvml was loaded, but no Nvidia GPU could be detected: {:#}",
                err
            ),
        }
    } else {
        debug!("Nvml was not loaded: {:#}", nvml_res.unwrap_err());
    }

    if gpus.is_empty() {
        anyhow::bail!("Could not detect any GPU");
    }
    info!("Initialized GPUs: {}", gpus_to_string(&gpus));

    let fan_controller = FanController::new(
        TARGET_TEMPERATURE.try_into().unwrap(),
        FAN_ON_TEMPERATURE.try_into().unwrap(),
        FAN_OFF_TEMPERATURE.try_into().unwrap(),
        MIN_FAN_SPEED.try_into().unwrap(),
    );
    let mut fan_controllers = vec![fan_controller; gpus.len()];

    info!("Fan control started");

    'outer: loop {
        for (gpu, fan_controller) in gpus.iter_mut().zip(fan_controllers.iter_mut()) {
            let temperature = match gpu.read_temperature() {
                Ok(t) => t,
                Err(err) => {
                    error!("Failed to read temperature on '{}': {}", gpu.name(), err);
                    break 'outer;
                }
            };

            let target = fan_controller.eval(temperature);

            if let Err(err) = gpu.set_fan_speed(target) {
                error!("Failed to set fan speed on '{}': {}", gpu.name(), err);
                break 'outer;
            }
        }

        sleep(Duration::from_millis(UPDATE_PERIOD));
    }

    info!("Terminating program due to critical error");
    restore_default_policies(&mut gpus);
    anyhow::bail!("Program terminated due to critical error");
}

fn restore_default_policies(gpus: &mut [Box<dyn GPUInterface + '_>]) {
    for gpu in gpus.iter_mut() {
        if let Err(err) = gpu.restore_default_policy() {
            warn!(
                "Failed to restore default fan control on '{}': {}",
                gpu.name(),
                err
            )
        } else {
            info!("Restored default fan control on {}", gpu.name());
        }
    }
}
