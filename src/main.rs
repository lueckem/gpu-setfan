use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use clap::Parser;
use nvml_wrapper::Nvml;
use tracing::{debug, error, info, warn};

use crate::{
    cli::Cli,
    fan_controller::FanController,
    interface::{GPUInterface, gpus_to_string},
    nvidia::initialize_nvidia_gpus,
};

mod cli;
mod fan_controller;
mod fanspeed;
mod interface;
mod logging;
mod nvidia;
mod pi_controller;
mod temperature;

const UPDATE_PERIOD: u64 = 1000; // in ms

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    if let Err(err) = logging::init_logging() {
        println!("WARNING: Failed to setup logging: {:#}", err);
    }
    info!("Program started");
    let fan_controller = FanController::try_from(args)?;

    // setup ctrl-c signal handling
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    if let Err(err) = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }) {
        warn!("Failed to set ctrl-c signal handler: {}", err);
    }

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
        anyhow::bail!("could not detect any GPUs with controllable fans");
    }
    gpus.shrink_to_fit();
    info!("Initialized GPUs: {}", gpus_to_string(&gpus));

    // initialize fan controllers
    let mut fan_controllers = vec![fan_controller; gpus.len()];

    info!("Fan control started");
    while running.load(Ordering::SeqCst) {
        for (gpu, fan_controller) in gpus.iter_mut().zip(fan_controllers.iter_mut()) {
            let temperature = match gpu.read_temperature() {
                Ok(t) => t,
                Err(err) => {
                    error!("Failed to read temperature on '{}': {:#}", gpu.name(), err);
                    error!("Terminating program due to critical error");
                    restore_default_policies(&mut gpus);
                    anyhow::bail!("program terminated due to critical error");
                }
            };

            let target = fan_controller.eval(temperature);

            if let Err(err) = gpu.set_fan_speed(target) {
                error!("Failed to set fan speed on '{}': {:#}", gpu.name(), err);
                error!("Terminating program due to critical error");
                restore_default_policies(&mut gpus);
                anyhow::bail!("program terminated due to critical error");
            }
        }

        sleep(Duration::from_millis(UPDATE_PERIOD));
    }

    info!("Received termination signal. Exiting program...");
    restore_default_policies(&mut gpus);
    Ok(())
}

fn restore_default_policies(gpus: &mut [Box<dyn GPUInterface + '_>]) {
    for gpu in gpus.iter_mut() {
        if let Err(err) = gpu.restore_default_policy() {
            warn!(
                "Failed to restore default fan control on '{}': {:#}",
                gpu.name(),
                err
            )
        } else {
            info!("Restored default fan control on '{}'", gpu.name());
        }
    }
}
