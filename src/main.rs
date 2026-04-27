use tracing::info;

mod fanspeed;
mod logging;

fn main() {
    logging::init_logging();

    info!("Program started");
}
