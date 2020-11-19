use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::{self, copy, hard_link, rename};

#[derive(Deserialize)]
struct AssetRelocationDef {
    variables_in_use: Vec<String>,
    jobs: Vec<JobConfigs>,
}

#[derive(Deserialize)]
struct JobConfigs {
    todo: String,
    src: String,
    dst: String,
}

fn parse_json() -> AssetRelocationDef {
    let json_text = fs::read_to_string("asset_relocation_def.json").expect("couldn't read file");
    let json_data: AssetRelocationDef =
        serde_json::from_str(&json_text).expect("json file format doesn't comply");
    json_data
}

fn map_args(json_data: &AssetRelocationDef) {
    let args: Vec<_> = env::args().into_iter().collect();
    assert_eq!(
        json_data.variables_in_use.len(),
        (args.len() - 1),
        "incorrect number of args passed"
    );
    let mut args_map: HashMap<String, String> = HashMap::new();
    for (i, val) in json_data.variables_in_use.iter().enumerate() {
        let val_arg = &val[1..(val.len() - 1)];
        args_map.insert(String::from(val_arg), args[i].clone());
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_parsing() {
        let value = parse_json();
        let value_iter = value.variables_in_use.iter();
        assert_eq!(
            *value_iter.peekable().peek().unwrap(),
            &String::from("{Configuration}")
        );
        let value_iter = value.variables_in_use.iter();
        assert_eq!(
            *value_iter.rev().peekable().peek().unwrap(),
            &String::from("{MonacoBinDir}")
        );
        let value_iter = value.jobs.iter();
        assert_eq!(
            &value.jobs[6].dst,
            "{MonacoBinDir}/{Configuration}/Games/{ProjectName}"
        );
        assert_eq!(&value.jobs[5].src, "{ProjectDir}/host.cmdline");
    }
}
