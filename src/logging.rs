#[cfg(target_os = "linux")]
pub fn init_logging() -> anyhow::Result<()> {
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
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn init_logging() -> anyhow::Result<()> {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::prelude::*;

    if cfg!(debug_assertions) {
        // Debug build
        tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        // Release build
        // On windows, the program has to maintain the log files
        std::fs::create_dir_all("C:/ProgramData/gpu-setfan/logs")?;
        let file_appender = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("gpu-setfan")
            .filename_suffix("log")
            .max_log_files(8)
            .build("C:/ProgramData/gpu-setfan/logs")?;

        let layer_stdout = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout.with_max_level(tracing::Level::INFO))
            .with_target(false);
        let layer_file = tracing_subscriber::fmt::layer()
            .with_writer(file_appender.with_max_level(tracing::Level::INFO))
            .with_target(false)
            .with_ansi(false);
        let subscriber = tracing_subscriber::Registry::default()
            .with(layer_stdout)
            .with(layer_file);
        tracing::subscriber::set_global_default(subscriber)?;
    }
    Ok(())
}
