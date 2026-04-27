use tracing::info;

mod fanspeed;
mod logging;
mod temperature;

fn main() {
    logging::init_logging();

    info!("Program started");
}
