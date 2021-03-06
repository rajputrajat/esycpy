use anyhow::Result;
use git_version::git_version;
use log::trace;
use simplelog::*;
use std::fs::File;

mod args;
mod json_parser;
mod operations;

use args::{get_args, ArgsType};
use json_parser::get_json_args;
use operations::FileOp;

fn main() -> Result<()> {
    print_git_version();
    setup_logger()?;
    log_panics::init();
    let args = get_args();
    eprintln!("input: {:?}", args);
    trace!("{:#?}", args);
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
            File::create(".esycpy.log")?,
        ),
    ])?;
    Ok(())
}

fn print_git_version() {
    const GIT_VERSION: &str = git_version!();
    eprintln!("Running esycpy, version: {}", GIT_VERSION);
}
