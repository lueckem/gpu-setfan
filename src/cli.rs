use clap::Parser;

// TODO: description

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(default_value = "80", help = "in °C")]
    pub target_temperature: f64,

    #[arg(
        long,
        help = "Temperature in °C at which fans turn on (must be < target)"
    )]
    pub fan_on: Option<f64>,

    #[arg(
        long,
        help = "Temperature in °C at which fans turn off (must be < fan-on)"
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
