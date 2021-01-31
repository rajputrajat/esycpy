use clap::{Arg, App, SubCommand};

pub enum Operation {
    Copy_,
    Move,
    Hardlink
}

pub enum ArgsType {
    CmdLine {
        op: Operation,
        from: String,
        to: String,
    },
    Json {
        json_file: String,
        variables: Option<Vec<(String, String)>>
    }
}

pub fn get_args() {
    let arg_from = Arg::with_name("from")
        .short("s")
        .long("from")
        .takes_value(true)
        .value_name("source_path")
        .required(true);
    let arg_to = Arg::with_name("to")
        .short("d")
        .long("to")
        .takes_value(true)
        .value_name("destination_path")
        .required(true);
    let matches = App::new("EsyCpy")
        .version("0.1.0")
        .author("Rajat Rajput <rajputrajat@gmail.com")
        .about("copy, move files and create hardlinks with ease.")
        .subcommand(SubCommand::with_name("copy")
            .about("copy file/dir from source to destination")
            .arg(arg_from.clone())
            .arg(arg_to.clone()))
        .subcommand(SubCommand::with_name("move")
            .about("move file/dir from source to destination")
            .arg(arg_from.clone())
            .arg(arg_to.clone()))
        .subcommand(SubCommand::with_name("hardlink")
            .about( "create hardlinks of file/s from source to destination")
            .arg(arg_from)
            .arg(arg_to))
        .arg(Arg::with_name("json_file")
            .short("j")
            .long("json")
            .help("json input file path which defines copy/move/hardlink operations")
            .takes_value(true)
            .value_name("json_file_path")
            .required(true))
        .arg_from_usage("-v, --variables=<VARIABLE_NAME_VALUE_PAIR>... 'var name - value pairs'")
        .get_matches();
}
