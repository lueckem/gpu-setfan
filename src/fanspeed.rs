use thiserror::Error;

/// Fan speed between 0.0 and 1.0, or as integer percentage between 0 and 100.
///
/// Internally stored as f64 between 0.0 and 1.0,
/// but can convert to and from the percentage representation as u32.
#[derive(Debug)]
pub struct FanSpeed(f64);

impl TryFrom<f64> for FanSpeed {
    type Error = FanSpeedError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value < 0.0 || value > 1.0 {
            Err(FanSpeedError::OutOfRangeFloat(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<u32> for FanSpeed {
    type Error = FanSpeedError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 100 {
            Err(FanSpeedError::OutOfRangeInt(value))
        } else {
            let value = (value as f64) / 100.0;
            Ok(Self(value))
        }
    }
}

impl From<FanSpeed> for f64 {
    fn from(value: FanSpeed) -> Self {
        value.0
    }
}

impl From<FanSpeed> for u32 {
    fn from(value: FanSpeed) -> Self {
        (value.0 * 100.0).round() as u32
    }
}

#[derive(Error, Debug)]
pub enum FanSpeedError {
    #[error("fan speed has to be between 0.0 and 1.0, got {0}")]
    OutOfRangeFloat(f64),
    #[error("fan speed has to be between 0 and 100, got {0}")]
    OutOfRangeInt(u32),
}
