use thiserror::Error;

/// GPU temperature in °C.
///
/// Stored as f64 between 0.0 and 120.0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct GPUTemperature(f64);

impl GPUTemperature {
    /// Retrieve the inner value (f64 between 0.0 and 120.0)
    pub fn inner(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for GPUTemperature {
    type Error = GPUTemperatureError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value < 0.0 || value > 120.0 {
            Err(GPUTemperatureError::OutOfRange(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl From<GPUTemperature> for f64 {
    fn from(value: GPUTemperature) -> Self {
        value.0
    }
}

#[derive(Error, Debug)]
pub enum GPUTemperatureError {
    #[error("invalid temperature {0}°C")]
    OutOfRange(f64),
}
