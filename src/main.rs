use serde::Deserialize;
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

fn do_jobs() {
    let json_text = fs::read_to_string("asset_relocation_def.json").expect("couldn't read file");
    let json_data: AssetRelocationDef =
        serde_json::from_str(&json_text).expect("json file format doesn't comply");
    println!("value: {}", json_data.variables_in_use[3]);
}

fn main() {
    do_jobs();
}
