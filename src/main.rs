use simplelog::*;
use std::fs::File;
use failure::Fallible;

mod args;
mod json_parser;
mod operations;

fn main() -> Fallible<()> {
    setup_logger()?;
    log_panics::init();
    let args = args::get_args();
    println!("{:#?}", args);
    Ok(())
}

fn setup_logger() -> Fallible<()> {
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

