use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use anyhow::bail;
use clap::Parser;
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

// TODO: description

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(default_value = "80", help = "in °C")]
    target_temperature: Option<f64>,

    #[arg(long, help = "Temperature in °C at which fans turn on")]
    fan_on: Option<f64>,

    #[arg(long, help = "Temperature in °C at which fans turn off")]
    fan_off: Option<f64>,

    #[arg(
        long,
        default_value = "30",
        help = "Minimum fan speed in % (between 0 and 100)"
    )]
    min_speed: Option<u32>,
}

fn validate_args(args: Args) -> anyhow::Result<FanController> {
    let target_temperature = args.target_temperature.unwrap();
    let fan_on_temperature = args.fan_on.unwrap_or(target_temperature - 10.0);
    let fan_off_temperature = args.fan_off.unwrap_or(fan_on_temperature - 5.0);
    let min_fan_speed = args.min_speed.unwrap();

    // TODO: more checks and testing
    // TODO: warn if parameters are odd? E.g., min_fan_speed=90
    if target_temperature < 30.0 || target_temperature > 100.0 {
        bail!("Invalid target temperature {target_temperature}! Choose a value between 30 and 100");
    }
    if fan_on_temperature >= target_temperature {
        bail!("The fan-on temperature has to be smaller than the target temperature");
    }
    if fan_off_temperature >= fan_on_temperature {
        bail!("The fan-off temperature has to be smaller than the fan-on temperature");
    }
    if fan_on_temperature < 20.0 {
        bail!("Invalid fan-on temperature {fan_on_temperature}! Choose a value larger than 20");
    }
    if fan_off_temperature < 15.0 {
        bail!("Invalid fan-off temperature {fan_on_temperature}! Choose a value larger than 15");
    }
    if min_fan_speed > 100 {
        bail!("Invalid minimum fan speed {min_fan_speed}! Choose a value between 0 and 100");
    }

    Ok(FanController::new(
        target_temperature.try_into().unwrap(),
        fan_on_temperature.try_into().unwrap(),
        fan_off_temperature.try_into().unwrap(),
        min_fan_speed.try_into().unwrap(),
    ))
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let fan_controller = validate_args(args)?;

    logging::init_logging();
    info!("Program started");
    // TODO: log the parsed parameters

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
        anyhow::bail!("Could not detect any GPU");
    }
    info!("Initialized GPUs: {}", gpus_to_string(&gpus));

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
                    anyhow::bail!("Program terminated due to critical error");
                }
            };

            let target = fan_controller.eval(temperature);

            if let Err(err) = gpu.set_fan_speed(target) {
                error!("Failed to set fan speed on '{}': {:#}", gpu.name(), err);
                error!("Terminating program due to critical error");
                restore_default_policies(&mut gpus);
                anyhow::bail!("Program terminated due to critical error");
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
