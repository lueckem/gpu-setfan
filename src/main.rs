use tracing::info;

mod logging;

fn main() {
    logging::init_logging();

    info!("Program started");
}
