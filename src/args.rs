use clap::{App, Arg, SubCommand};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum Operation {
    Copy_,
    Move,
    Hardlink,
}

#[derive(Debug, PartialEq)]
pub enum ArgsType {
    CmdLine {
        op: Operation,
        from: PathBuf,
        to: PathBuf,
    },
    Json {
        json_file: PathBuf,
        variables: Option<Vec<(String, String)>>,
    },
}

pub fn get_args() -> ArgsType {
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
    let app = App::new("EsyCpy")
        .author("Rajat Rajput <rajputrajat@gmail.com")
        .about("copy, move files and create hardlinks with ease.")
        .help(HELP)
        .subcommand(
            SubCommand::with_name("copy")
                .about("copy file/dir from source to destination")
                .arg(arg_from.clone())
                .arg(arg_to.clone()),
        )
        .subcommand(
            SubCommand::with_name("move")
                .about("move file/dir from source to destination")
                .arg(arg_from.clone())
                .arg(arg_to.clone()),
        )
        .subcommand(
            SubCommand::with_name("hardlink")
                .about("create hardlinks of file/s from source to destination")
                .arg(arg_from)
                .arg(arg_to),
        )
        .arg(
            Arg::with_name("json_file")
                .short("j")
                .long("json")
                .takes_value(true)
                .validator(|p| -> Result<(), String> {
                    let jfile = Path::new(p.as_str()).canonicalize();
                    if jfile.is_ok() {
                        Ok(())
                    } else {
                        Err(String::from("json file path isn't correct."))
                    }
                })
                .value_name("JSON_FILE_PATH"),
        )
        .arg(
            Arg::with_name("variables")
                .short("v")
                .long("variables")
                .requires("json_file")
                .min_values(1)
                .validator(|v| -> Result<(), String> {
                    if v.contains('=') {
                        Ok(())
                    } else {
                        Err(String::from("assignment operator not found. check help."))
                    }
                })
                .value_name("VARIABLE_NAME_VALUE_PAIR"),
        );
    let matches = app.clone().get_matches();
    let json_file_path = matches.value_of("json_file");
    if let Some(json_file_path) = json_file_path {
        let json_file = Path::new(json_file_path).to_owned();
        if let Some(variables) = matches.values_of("variables") {
            let variables = variables.fold(Vec::new(), |mut vec: Vec<(String, String)>, v| {
                let name_value: Vec<&str> = (*v).split('=').collect();
                vec.push((String::from(name_value[0]), String::from(name_value[1])));
                vec
            });
            ArgsType::Json {
                json_file,
                variables: Some(variables),
            }
        } else {
            ArgsType::Json {
                json_file,
                variables: None,
            }
        }
    } else {
        let subcommand = match matches.subcommand_name() {
            Some("copy") => ("copy", Operation::Copy_),
            Some("move") => ("move", Operation::Move),
            Some("hardlink") => ("hardlink", Operation::Hardlink),
            _ => {
                eprintln!("no arguments given. run \"esycpy --help\" to learn it's usage");
                std::process::exit(0);
            }
        };
        if let Some(subcommand_matches) = matches.subcommand_matches(subcommand.0) {
            let source = subcommand_matches.value_of("from").unwrap();
            let destination = subcommand_matches.value_of("to").unwrap();
            ArgsType::CmdLine {
                op: subcommand.1,
                from: PathBuf::from(source),
                to: PathBuf::from(destination),
            }
        } else {
            panic!("neither json option is provided, nor cmdline option is used. use help command");
        }
    }
}

const HELP: &str = r#"
EsyCpy
Rajat Rajput <rajputrajat@gmail.com>
copy, move and create hardlinks of files/dirs with ease

USAGE:
    esycpy [SUBCOMMAND]
    esycpy [OPTIONS]

FLAGS:
    -h, --help      Prints help information
    -V, --version   Prints version information

OPTIONS:
    -j, --json <JSON_FILE_PATH>
                    Json file path which defines copy/move/hardlink operations
    -v, --variables <VARIABLE_NAME_VALUE_PAIR>...
                    these are optionally used in input json file,
                    multiple values can be given like this <var_name=var_value>

SUBCOMMANDS:
    copy            copy file/dir from source to destination
    hardlink        create hardlinks of file/s from source to destination
    help            Prints this message or the help of the given subcommand(s)
    move            move file/dir from source to destination

EXAMPLES:
    USING INPUT JSON FILE:
    > esycpy -j /home/example/asset_copier.json
    ------------------------- ASSET_COPIER.JSON -----------------------------
    |                                                                       |
    |    "variables_in_use": [                                              |
    |    ],                                                                 |
    |    "jobs": [                                                          |
    |        {                                                              |
    |            "todo": "hardlink",                                        |
    |            "src": "c:/Users/example/src_dir",                         |
    |            "dst": "c:/Users/example/desktop/here"                     |
    |        },                                                             |
    |        {                                                              |
    |            "todo": "hardlink",                                        |
    |            "src": "c:/Users/example/src_dir/*",                       |
    |            "dst": "c:/Users/example/documents/hlinks_all_here"        |
    |        },                                                             |
    |    ]                                                                  |
    |}                                                                      |
    -------------------------------------------------------------------------

    > esycpy -j /home/example/copier.json -v songs_dir=/home/example/songs pdfs=/home/example/study docs=/home/example/documents
    --------------------------- COPIER.JSON ---------------------------------
    |{                                                                      |
    |    "variables_in_use": [                                              |
    |        "{songs_dir}",                                                 |
    |        "{pdfs}",                                                      |
    |        "{docs}"                                                       |
    |    ],                                                                 |
    |    "jobs": [                                                          |
    |        {                                                              |
    |            "todo": "hardlink",                                        |
    |            "src": "{songs_dir}/**.ogg",                               |
    |            "dst": "/home/example/all_oggs"                            |
    |        },                                                             |
    |        {                                                              |
    |            "todo": "move",                                            |
    |            "src": "{pdfs},                                            |
    |            "dst": "/home/example/to_new_dir"                          |
    |        },                                                             |
    |        {                                                              |
    |            "todo": "copy",                                            |
    |            "src": "{docs}/*.docx",                                    |
    |            "dst": "/home/example/this_dir_docx_files"                 |
    |        },                                                             |
    |    ]                                                                  |
    |}                                                                      |
    -------------------------------------------------------------------------

    USING OPTIONS:
    1. Create hardlink of file to new_hard_link
        > esycpy hardlink -s c:/users/example/file -d c:/users/example/new_hard_link
    2. Create hardlink of dir recursively
        > esycpy hardlink -s c:/users/example/dir1 -d c:/users/example/dir_with_hlinks
    3. Move xml files from this dir to destination dir
        > esycpy move -s c:/users/example/dir2/*.xml -d c:/users/example/dir_move_in_here
    4. Copy all ogg files recursively to destination dir
        > esycpy copy /home/example/audios/**.ogg /home/example/only_oggs"#;
