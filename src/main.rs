use anyhow::Result;
use simplelog::*;
use std::fs::File;

mod args;
mod json_parser;
mod operations;

fn main() -> Result<()> {
    setup_logger()?;
    log_panics::init();
    let args = args::get_args();
    println!("{:#?}", args);
    Ok(())
}

fn setup_logger() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create("asset_maker.log")?,
        ),
    ])?;
    Ok(())
}
