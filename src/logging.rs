use log::info;
use tui_logger::{
    TuiLoggerFile, TuiLoggerLevelOutput, init_logger, set_default_level, set_log_file,
};

pub fn init_logging() -> color_eyre::Result<()> {
    init_logger(log::LevelFilter::Trace)?;
    set_default_level(log::LevelFilter::Trace);

    let mut log_file = std::env::temp_dir();

    log_file.push("/tmp/eilmeldung.log"); // TODO make configurable

    let file_options = TuiLoggerFile::new(log_file.to_str().unwrap())
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_file(false)
        .output_separator(':');
    set_log_file(file_options);

    info!("initialized logging");
    info!("logging to file {}", log_file.to_str().unwrap());

    Ok(())
}
