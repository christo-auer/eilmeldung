use crate::prelude::*;
use log::info;
use tui_logger::{
    TuiLoggerFile, TuiLoggerLevelOutput, init_logger, set_default_level, set_log_file,
};

pub fn init_logging(cli_args: &CliArgs) -> color_eyre::Result<()> {
    init_logger(log::LevelFilter::Trace)?;

    set_default_level(cli_args.log_level().unwrap_or(log::LevelFilter::Info));

    if let Some(log_file) = cli_args.log_file() {
        let file_options = TuiLoggerFile::new(log_file)
            .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
            .output_file(false)
            .output_separator(':');
        set_log_file(file_options);
        info!("logging to file {}", log_file);
    }

    info!("initialized logging");

    Ok(())
}
