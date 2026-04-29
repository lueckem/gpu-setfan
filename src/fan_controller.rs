use std::time::Instant;

use tracing::debug;

use crate::{fanspeed::FanSpeed, pi_controller::PIController, temperature::GPUTemperature};

/// Fan controller using a PI controller.
///
/// The behavior is as follows:
/// - The fans get turned on when temperature exceeds `fan_on_temperature`.
/// - Then the fan speed is controlled with a PI controller aiming at `target_temperature`. The fan speed is always larger than the `min_fan_speed`.
/// - If the temperature falls below the `fan_off_temperature`, the fans will get turned off again.
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
