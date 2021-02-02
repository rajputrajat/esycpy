use crate::args::ArgsType;
use log::debug;
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub fn get_json_args(args: ArgsType) -> Vec<ArgsType> {
    match args {
        ArgsType::Json {
            json_file,
            variables,
        } => {
            let json_def = parse_json(Path::new(&json_file).as_ref());
            Vec::new()
        }
        _ => unreachable!(),
    }
}

#[derive(Deserialize)]
pub struct AssetRelocationDef {
    variables_in_use: Vec<String>,
    jobs: Vec<JobConfigs>,
}

#[derive(Deserialize, Debug)]
struct JobConfigs {
    todo: String,
    src: String,
    dst: String,
}

fn parse_json(path: &Path) -> AssetRelocationDef {
    let json_text = fs::read_to_string(path).expect("couldn't read file");
    debug!("{} file is read", path.to_str().unwrap());
    let json_data: AssetRelocationDef =
        serde_json::from_str(&json_text).expect("json file format doesn't comply");
    debug!("json file is parsed");
    json_data
}
