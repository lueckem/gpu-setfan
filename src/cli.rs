use clap::Parser;

const DESCRIPTION: &str = "GPU fan control via temperature setpoint.

Automatically adjusts fan speed to keep your GPU at the specified target temperature.
The behavior is as follows:
- At the start, the fans are off (speed = 0%).
- The fans get turned on when the temperature exceeds the <FAN_ON> temperature.
- Then the fan speed is controlled with a PI controller to maintain the [TARGET_TEMPERATURE]. The fan speed is always larger than <MIN_SPEED> to ensure stable operation.
- When the temperature falls below the <FAN_OFF> temperature, the fans get turned off again.";

#[derive(Parser, Debug)]
#[command(version, about=DESCRIPTION)]
pub struct Cli {
    #[arg(default_value = "80", help = "in °C")]
    pub target_temperature: f64,

    #[arg(
        long,
        help = "Temperature in °C at which fans turn on; must be < target [default: target - 10]"
    )]
    pub fan_on: Option<f64>,

    #[arg(
        long,
        help = "Temperature in °C at which fans turn off; must be < fan-on [default: fan-on - 5]"
    )]
    pub fan_off: Option<f64>,

    #[arg(
        long,
        default_value = "30",
        help = "Minimum fan speed in % (0-100)",
        value_parser = clap::value_parser!(u32).range(0..=100),
    )]
    pub min_speed: u32,
}
