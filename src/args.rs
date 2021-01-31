use clap::{Arg, ArgGroup, App, SubCommand};

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

pub fn get_args() {
    let arg_from = Arg::with_name("from")
        .short("s")
        .long("from")
        .value_name("source_path")
        .required(true);
    let arg_to = Arg::with_name("to")
        .short("d")
        .long("to")
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
        .get_matches();
}
