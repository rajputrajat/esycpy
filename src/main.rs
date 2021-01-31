use simplelog::*;
use std::fs::File;

mod args;
mod json_parser;

fn main() {
    setup_logger();
    log_panics::init();
    let args = args::get_args();
    println!("{:#?}", args);
}

fn setup_logger() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create("asset_maker.log").unwrap(),
        ),
    ])
    .unwrap();
}

