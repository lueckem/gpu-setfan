/// PI Controller with smoothing.
///
/// Based on the error `err` between process variable and setpoint, the control `u` is
/// `u = proportional_gain * err + integral_gain * integral`,
/// where the `integral` is over the past errors.
///
/// The computed control `u` is clamped in `[-1, 1]` and then smoothed via
/// `u = smoothing_factor * u + (1 - smoothing factor) * last_u`.
#[derive(Clone)]
pub struct PIController {
    proportional_gain: f64,
    integral_gain: f64,
    integral: f64,
    last_u: Option<f64>,
    smoothing_factor: f64,
}

impl PIController {
    pub fn new(proportional_gain: f64, integral_gain: f64, smoothing_factor: f64) -> Self {
        Self {
            proportional_gain,
            integral_gain,
            integral: 0.0,
            last_u: None,
            smoothing_factor,
        }
    }

    /// Update the PI controller.
    ///
    /// `delta_t` is the time since the last update in seconds.
    pub fn update(&mut self, err: f64, delta_t: f64) -> f64 {
        if self.integral_gain > 0.0 {
            self.integral += err * delta_t;

            // clamp the integral term in [-1, 1] so it does not wind up too much
            self.integral = self
                .integral
                .clamp(-1.0 / self.integral_gain, 1.0 / self.integral_gain);
        }

        let u = self.proportional_gain * err + self.integral_gain * self.integral;
        let u = u.clamp(-1.0, 1.0);
        let last_u = self.last_u.unwrap_or(u);
        let u = self.smoothing_factor * u + (1.0 - self.smoothing_factor) * last_u;
        self.last_u = Some(u);
        u
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.last_u = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(x: f64, y: f64) -> bool {
        let rtol = 1e-10;
        (x - y).abs() <= rtol * x.abs().max(y.abs())
    }

    #[test]
    fn proportional_with_error() {
        let mut controller = PIController::new(2.0, 0.0, 1.0);
        let u = controller.update(0.3, 1.0);
        assert!(approx_eq(u, 0.6));
    }

    #[test]
    fn proportional_with_negative_error() {
        let mut controller = PIController::new(2.0, 0.0, 1.0);
        let u = controller.update(-0.3, 1.0);
        assert!(approx_eq(u, -0.6));
    }

    #[test]
    fn output_clamped() {
        let mut controller = PIController::new(1.0, 0.0, 1.0);
        let u = controller.update(10.0, 1.0);
        assert!(approx_eq(u, 1.0));
        let u = controller.update(-10.0, 1.0);
        assert!(approx_eq(u, -1.0));
    }

    #[test]
    fn integral_accumulates() {
        let mut controller = PIController::new(0.0, 0.5, 1.0);
        let u = controller.update(0.1, 1.0);
        // 0.5 * (0.1 * 1.0)
        assert!(approx_eq(u, 0.05));

        // 0.5 * (0.1 * 1.0 + 0.1 * 2.0)
        let u = controller.update(0.1, 2.0);
        assert!(approx_eq(u, 0.15));
    }

    #[test]
    fn integral_wind_up_pos() {
        let mut controller = PIController::new(0.0, 1.0, 1.0);
        for _ in 0..100 {
            controller.update(0.1, 1.0);
        }
        let u = controller.update(0.0, 1.0);
        assert_eq!(u, 1.0);
    }

    #[test]
    fn integral_wind_up_neg() {
        let mut controller = PIController::new(0.0, 1.0, 1.0);
        for _ in 0..100 {
            controller.update(-0.2, 1.0);
        }
        let u = controller.update(0.0, 1.0);
        assert_eq!(u, -1.0);
    }

    #[test]
    fn smoothing() {
        let mut controller = PIController::new(1.0, 0.0, 0.5);
        let u = controller.update(0.1, 1.0);
        assert!(approx_eq(u, 0.1));
        let u = controller.update(0.2, 1.0);
        // 0.2 * 0.5 + 0.1 * 0.5
        assert!(approx_eq(u, 0.15));
    }
}
