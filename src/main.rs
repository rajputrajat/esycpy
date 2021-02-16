use anyhow::Result;
use simplelog::*;
use std::fs::File;

mod args;
mod json_parser;
mod operations;

use args::{get_args, ArgsType};
use json_parser::get_json_args;
use operations::FileOp;

fn main() -> Result<()> {
    setup_logger()?;
    log_panics::init();
    let args = get_args();
    println!("{:#?}", args);
    match args {
        ArgsType::CmdLine {
            op: _,
            from: _,
            to: _,
        } => {
            let file_op = FileOp::from(args);
            file_op.process()?;
        }
        ArgsType::Json {
            json_file: _,
            variables: _,
        } => {
            let v_args = get_json_args(args);
            for args in v_args {
                let file_op = operations::FileOp::from(args);
                file_op.process()?;
            }
        }
    }
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
