use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;

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

fn parse_json(path: &str) -> AssetRelocationDef {
    let json_text = fs::read_to_string(path).expect("couldn't read file");
    let json_data: AssetRelocationDef =
        serde_json::from_str(&json_text).expect("json file format doesn't comply");
    json_data
}

fn map_args(json_data: &AssetRelocationDef, args: &Vec<&str>) -> HashMap<String, String> {
    assert_eq!(
        json_data.variables_in_use.len(),
        args.len(),
        "incorrect number of args passed"
    );
    let mut args_map: HashMap<String, String> = HashMap::new();
    for (i, val) in json_data.variables_in_use.iter().enumerate() {
        args_map.insert(String::from(val), String::from(args[i]));
    }
    args_map
}

struct FileHandlers {
    hardlink: fn(src: &str, dst: &str) -> Option<String>,
    cpy: fn(src: &str, dst: &str) -> Option<String>,
    mov: fn(src: &str, dst: &str) -> Option<String>,
}

impl FileHandlers {
    fn new() -> Self {
        FileHandlers {
            hardlink: |s, d| {
                fs::hard_link(s, d).unwrap();
                None
            },
            cpy: |s, d| {
                let _ = fs::copy(s, d).unwrap();
                None
            },
            mov: |s, d| {
                fs::rename(s, d).unwrap();
                None
            },
        }
    }
}

fn do_jobs(json_data: &AssetRelocationDef, args_map: &HashMap<String, String>) {
    for job in &json_data.jobs {
        let file_handlers = FileHandlers::new();
        let _ = do_job(job, &file_handlers, args_map);
    }
}

fn do_job(
    job: &JobConfigs,
    file_handlers: &FileHandlers,
    args_map: &HashMap<String, String>,
) -> Option<String> {
    let mut src = job.src.clone();
    let mut dst = job.dst.clone();
    for (arg, val) in args_map {
        src = src.replace(arg, val.as_str());
        dst = dst.replace(arg, val);
    }
    match job.todo.as_str() {
        "hardlink" => (file_handlers.hardlink)(src.as_str(), dst.as_str()),
        "copy" => (file_handlers.cpy)(src.as_str(), dst.as_str()),
        "move" => (file_handlers.mov)(src.as_str(), dst.as_str()),
        _ => panic!("this is new. not yet handled"),
    }
}

fn main() {
    //"asset_relocation_def.json"
    let args: Vec<_> = env::args().skip(1).collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_parsing() {
        let value = parse_json("asset_relocation_def.json");
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
        assert_eq!(
            &value.jobs[6].dst,
            "{MonacoBinDir}/{Configuration}/Games/{ProjectName}"
        );
        assert_eq!(&value.jobs[5].src, "{ProjectDir}/host.cmdline");
    }

    #[test]
    fn check_args_map_good() {
        let args = vec!["one", "two", "three", "four", "five", "six", "seven"];
        let json_data = parse_json("asset_relocation_def.json");
        let args_map = map_args(&json_data, &args);
        assert_eq!(args_map["{Configuration}"], "one");
        assert_eq!(args_map["{MonacoBinDir}"], "seven");
        assert_eq!(args_map["{ProjectDir}"], "four");
    }

    #[test]
    fn which_operation() {
        let args = vec!["one", "two", "three", "four", "five", "six", "seven"];
        let json_data = parse_json("asset_relocation_def.json");
        let args_map = map_args(&json_data, &args);
        let fn_handlers = FileHandlers {
            hardlink: |s, d| {
                let mut buf = String::new();
                write!(&mut buf, "hardlink, {}, {}", s, d).expect("couldn't write");
                Some(buf)
            },
            cpy: |s, d| {
                let mut buf = String::new();
                write!(&mut buf, "copy, {}, {}", s, d).expect("couldn't write");
                Some(buf)
            },
            mov: |s, d| {
                let mut buf = String::new();
                write!(&mut buf, "move, {}, {}", s, d).expect("couldn't write");
                Some(buf)
            },
        };
        assert_eq!(
            "hardlink, four/assets/setup.txt, six/setup.txt",
            do_job(&json_data.jobs[0], &fn_handlers, &args_map).unwrap()
        );
        assert_eq!(
            concat!(
                "hardlink, ",
                "six/../../../../Tools/GDKRuntimeHost/two/one, ",
                "seven/one/Runtime/bin"
            ),
            do_job(&json_data.jobs[4], &fn_handlers, &args_map).unwrap()
        );
        assert_eq!(
            "copy, , ",
            do_job(&json_data.jobs[7], &fn_handlers, &args_map).unwrap()
        );
    }
}
