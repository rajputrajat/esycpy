use log::{debug, error};
use serde::Deserialize;
use simplelog::*;
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct AssetRelocationDef {
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

fn map_args(json_data: &AssetRelocationDef, args: String) -> HashMap<String, String> {
    let args: Vec<String> = args.split(", ").map(|x| String::from(x)).collect();
    assert_eq!(
        json_data.variables_in_use.len(),
        args.len(),
        "incorrect number of args passed"
    );
    let mut args_map: HashMap<String, String> = HashMap::new();
    for (i, val) in json_data.variables_in_use.iter().enumerate() {
        args_map.insert(String::from(val), String::from(&args[i]));
    }
    debug!("args: {:?}", args_map);
    args_map
}

struct FileHandlers {
    hardlink: fn(src: &str, dst: &str) -> Option<String>,
    cpy: fn(src: &str, dst: &str) -> Option<String>,
    mov: fn(src: &str, dst: &str) -> Option<String>,
}

impl FileHandlers {
    fn new() -> Self {
        debug!("preparing file handlers");
        FileHandlers {
            hardlink: |s, d| {
                debug!("creating hardlink from '{}' to '{}'", s, d);
                fs::hard_link(s, d).expect("couldn't create hardlink");
                None
            },
            cpy: |s, d| {
                debug!("copying from '{}' to '{}'", s, d);
                let _ = fs::copy(s, d).expect("problem while copying");
                None
            },
            mov: |s, d| {
                debug!("moving from '{}' to '{}'", s, d);
                fs::rename(s, d).expect("move operation failed");
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
    debug!("current job is: {:?}", job);
    let mut src = job.src.clone();
    let mut dst = job.dst.clone();
    for (arg, val) in args_map {
        src = src.replace(arg, val.as_str());
        dst = dst.replace(arg, val);
    }
    let dst_path: &Path = Path::new(dst.as_str());
    if dst_path.exists() {
        fs::remove_file(dst_path).expect("error in file deletion");
    }
    match job.todo.as_str() {
        "hardlink" => (file_handlers.hardlink)(src.as_str(), dst.as_str()),
        "copy" => (file_handlers.cpy)(src.as_str(), dst.as_str()),
        "move" => (file_handlers.mov)(src.as_str(), dst.as_str()),
        _ => panic!("this is new. not yet handled"),
    }
}

fn setup_logger() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            fs::File::create("asset_maker.log").unwrap(),
        ),
    ])
    .unwrap();
}

fn main() {
    //"asset_relocation_def.json"
    setup_logger();
    let json_data = parse_json(Path::new("asset_relocation_def.json"));
    let args = env::args().skip(1).next().unwrap();
    debug!("args: {:?}", args);
    let args_map = map_args(&json_data, args);
    do_jobs(&json_data, &args_map);
    debug!("all done");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_parsing() {
        let value = parse_json(Path::new("asset_relocation_def.json"));
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
        let args = "one, two, three, four, five, six, seven";
        let json_data = parse_json(Path::new("asset_relocation_def.json"));
        let args_map = map_args(&json_data, String::from(args));
        assert_eq!(args_map["{Configuration}"], "one");
        assert_eq!(args_map["{MonacoBinDir}"], "seven");
        assert_eq!(args_map["{ProjectDir}"], "four");
    }

    #[test]
    fn which_operation() {
        let args = "one, two, three, four, five, six, seven";
        let json_data = parse_json(Path::new("asset_relocation_def.json"));
        let args_map = map_args(&json_data, String::from(args));
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
