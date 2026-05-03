use tracing_subscriber;

// #[cfg(target_os = "linux")]
pub fn init_logging() {
    if cfg!(debug_assertions) {
        // Debug build
        tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        // Release build
        tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .without_time() // time is logged by systemd
            .init();
    }
}

// TODO: On windows, the program has to maintain the log file
//
// #[cfg(target_os = "windows")]
// fn init_logging() {
//     let file_appender = rolling::daily("C:/ProgramData/YourApp/logs", "app.log");
//     tracing_subscriber::fmt()
//         .with_writer(file_appender)
//         .init();
// }
