use tracing_subscriber;

// #[cfg(target_os = "linux")]
pub fn init_logging() {
    let max_level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    // TODO: disable timestamps in release build `.without_time()`
    // since timestamps will already be added by journald

    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_max_level(max_level)
        .init();
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
