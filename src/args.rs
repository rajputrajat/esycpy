use clap::{Arg, App, SubCommand};

pub enum ArgsType {
    CmdLine {
        from: String,
        to: String,
    },
    Json {
        json_file: String,
        variables: Option<Vec<String>>
    }
}

fn get_args() {}
