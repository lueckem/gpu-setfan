use std::time::Instant;

use anyhow::bail;
use tracing::{debug, info, warn};

use crate::{
    cli::Cli, fanspeed::FanSpeed, pi_controller::PIController, temperature::GPUTemperature,
};

/// Fan controller using a PI controller.
///
/// The behavior is as follows:
/// - The fans get turned on when temperature exceeds `fan_on_temperature`.
/// - Then the fan speed is controlled with a PI controller aiming at `target_temperature`. The fan speed is always larger than the `min_fan_speed`.
/// - If the temperature falls below the `fan_off_temperature`, the fans will get turned off again.
#[derive(Clone)]
pub struct FanController {
    pi_controller: PIController,
    target_temperature: GPUTemperature,
    fan_on_temperature: GPUTemperature,
    fan_off_temperature: GPUTemperature,
    min_fan_speed: FanSpeed,
    fans_off: bool,
    last_eval: Instant,
}

impl FanController {
    pub fn new(
        target_temperature: GPUTemperature,
        fan_on_temperature: GPUTemperature,
        fan_off_temperature: GPUTemperature,
        min_fan_speed: FanSpeed,
    ) -> Self {
        let proportional_gain = 0.1;
        let integral_gain = 0.01;
        let smoothing_factor = 0.5;
        Self {
            pi_controller: PIController::new(proportional_gain, integral_gain, smoothing_factor),
            target_temperature,
            fan_on_temperature,
            fan_off_temperature,
            min_fan_speed,
            fans_off: true,
            last_eval: Instant::now(),
        }
    }

    pub fn eval(&mut self, temp: GPUTemperature) -> FanSpeed {
        if self.fans_off && temp < self.fan_on_temperature {
            return FanSpeed::zero();
        }

        if !self.fans_off && temp <= self.fan_off_temperature {
            debug!("Fans turned off");
            self.fans_off = true;
            return FanSpeed::zero();
        }

        if self.fans_off {
            debug!("Fans turned on");
            self.fans_off = false;
            self.last_eval = Instant::now();
            self.pi_controller.reset();
        }

        let delta_t = self.last_eval.elapsed().as_secs_f64();
        let err = temp.inner() - self.target_temperature.inner();
        let u = self.pi_controller.update(err, delta_t);
        self.last_eval = Instant::now();
        self.convert_pi_to_fan_speed(u)
    }

    /// Maps value in [-1, 1] from PI controller to value in [u_0, 1],
    /// where 0 <= u_0 < 1 is the minimum fan speed
    fn convert_pi_to_fan_speed(&self, u: f64) -> FanSpeed {
        let u0: f64 = self.min_fan_speed.into();
        let u = u * (1.0 - u0) / 2.0 + (1.0 + u0) / 2.0;

        // this never panics since the input was in [-1, 1]
        u.try_into().unwrap()
    }
}

impl TryFrom<Cli> for FanController {
    type Error = anyhow::Error;

    fn try_from(args: Cli) -> Result<Self, Self::Error> {
        let target_temperature = args.target_temperature;
        let fan_on_temperature = args.fan_on.unwrap_or(target_temperature - 10.0);
        let fan_off_temperature = args.fan_off.unwrap_or(fan_on_temperature - 5.0);
        let min_fan_speed = args.min_speed;

        // check valid parameters
        if target_temperature < 30.0 || target_temperature > 100.0 {
            bail!(
                "invalid target temperature {target_temperature}°C: must be between 30°C and 100°C"
            );
        }
        if fan_on_temperature >= target_temperature {
            bail!(
                "fan-on temperature ({fan_on_temperature}°C) must be below target temperature ({target_temperature}°C)"
            );
        }
        if fan_off_temperature >= fan_on_temperature {
            bail!(
                "fan-off temperature ({fan_off_temperature}°C) must be below fan-on temperature ({fan_on_temperature}°C)"
            );
        }
        if fan_on_temperature < 20.0 {
            bail!("invalid fan-on temperature {fan_on_temperature}°C: must be at least 20°C");
        }
        if fan_off_temperature < 0.0 {
            bail!("invalid fan-off temperature {fan_off_temperature}°C: must be at least 0°C");
        }

        info!("Target temperature: {target_temperature}°C");
        info!("Fan-on temperature: {fan_on_temperature}°C");
        info!("Fan-off temperature: {fan_off_temperature}°C");
        info!("Minimum fan speed: {min_fan_speed}%");

        // warn on valid but odd parameters
        if target_temperature < 50.0 {
            warn!(
                "Target temperature {target_temperature}°C is rather low. Recommended: 65°C - 85°C"
            );
        }
        if target_temperature > 90.0 {
            warn!(
                "Target temperature {target_temperature}°C is very high and could cause thermal throttling. Recommended: 65°C - 85°C"
            );
        }
        if target_temperature - fan_on_temperature <= 5.0 {
            warn!(
                "Fan-on temperature is very close to target temperature, which may limit the controller's ability to react"
            )
        }
        if target_temperature - fan_off_temperature < 10.0 {
            warn!(
                "Fan-off temperature is very close to target temperature, which may reduce the controller's ability to operate"
            )
        }
        if fan_on_temperature - fan_off_temperature <= 3.0 {
            warn!(
                "Fan-on and fan-off temperature are very close, which may cause frequent cycling that negatively affects fan lifetime"
            )
        }
        if min_fan_speed >= 60 {
            warn!(
                "Minimum fan speed is very high. Recommended: set to lowest speed so that the fans reliably ramp up, typically around 30%"
            )
        }
        if min_fan_speed < 30 {
            warn!(
                "Low minimum fan speed. Check that the fans are able to reliably ramp up. If not, increase this value"
            )
        }

        // after the above validation all temperatures and the fan speed
        // are in the allowed range, so the below unwraps never panic
        Ok(Self::new(
            target_temperature.try_into().unwrap(),
            fan_on_temperature.try_into().unwrap(),
            fan_off_temperature.try_into().unwrap(),
            min_fan_speed.try_into().unwrap(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    fn controller_from_command(command: &[&str]) -> anyhow::Result<FanController> {
        let command = std::iter::once(&"").chain(command.iter());
        let cli = Cli::parse_from(command);
        FanController::try_from(cli)
    }

    #[test]
    fn from_command_default() {
        let controller = controller_from_command(&[]).unwrap();
        assert_eq!(controller.target_temperature.inner(), 80.0);
        assert_eq!(controller.fan_on_temperature.inner(), 70.0);
        assert_eq!(controller.fan_off_temperature.inner(), 65.0);
        assert_eq!(controller.min_fan_speed.inner(), 0.3);
    }

    #[test]
    fn from_command_target_temperature() {
        let controller = controller_from_command(&["75"]).unwrap();
        assert_eq!(controller.target_temperature.inner(), 75.0);
        assert_eq!(controller.fan_on_temperature.inner(), 65.0);
        assert_eq!(controller.fan_off_temperature.inner(), 60.0);
    }

    #[test]
    fn from_command_fan_on() {
        let controller = controller_from_command(&["--fan-on", "60"]).unwrap();
        assert_eq!(controller.target_temperature.inner(), 80.0);
        assert_eq!(controller.fan_on_temperature.inner(), 60.0);
        assert_eq!(controller.fan_off_temperature.inner(), 55.0);
    }

    #[test]
    fn from_command_fan_off() {
        let controller = controller_from_command(&["--fan-off", "60"]).unwrap();
        assert_eq!(controller.target_temperature.inner(), 80.0);
        assert_eq!(controller.fan_on_temperature.inner(), 70.0);
        assert_eq!(controller.fan_off_temperature.inner(), 60.0);
    }

    #[test]
    fn from_command_all() {
        let controller =
            controller_from_command(&["81", "--fan-on=68", "--fan-off=61", "--min-speed=33"])
                .unwrap();
        assert_eq!(controller.target_temperature.inner(), 81.0);
        assert_eq!(controller.fan_on_temperature.inner(), 68.0);
        assert_eq!(controller.fan_off_temperature.inner(), 61.0);
        assert_eq!(controller.min_fan_speed.inner(), 0.33);
    }

    #[test]
    fn from_command_min_speed() {
        let controller = controller_from_command(&["--min-speed", "40"]).unwrap();
        assert_eq!(controller.min_fan_speed.inner(), 0.4);
    }

    #[test]
    fn from_command_invalid_target() {
        assert!(controller_from_command(&["10"]).is_err());
        assert!(controller_from_command(&["105"]).is_err());
    }

    #[test]
    fn from_command_fan_on_too_large() {
        assert!(controller_from_command(&["70", "--fan-on", "75"]).is_err());
    }

    #[test]
    fn from_command_fan_on_too_small() {
        assert!(controller_from_command(&["70", "--fan-on", "10"]).is_err());
    }

    #[test]
    fn from_command_fan_off_too_large() {
        assert!(controller_from_command(&["80", "--fan-on=70", "--fan-off=75"]).is_err());
    }

    #[test]
    fn from_command_fan_off_too_small() {
        assert!(controller_from_command(&["80", "--fan-on=70", "--fan-off=-1"]).is_err());
    }
}
