use log::debug;
use log_panics;
use serde::Deserialize;
use simplelog::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

trait FileOperations {
    fn create_hardlink(&self);
    fn create_copy(&self);
    fn create_move(&self);
    fn remove_dst(&self);
    fn create_dst_dir(&self);
    fn common(&self) -> bool;
}

struct AssetPaths {
    src: PathBuf,
    dst: PathBuf,
}

impl FileOperations for AssetPaths {
    fn create_hardlink(&self) {
        if self.common() {
            debug!(
                "creating hardlink from '{:?}' to '{:?}'",
                self.src, self.dst
            );
            fs::hard_link(&self.src, &self.dst).expect("couldn't create hardlink");
        }
    }

    fn create_copy(&self) {
        if self.common() {
            debug!("copying from '{:?}' to '{:?}'", self.src, self.dst);
            let _ = fs::copy(&self.src, &self.dst).expect("problem while copying");
        }
    }

    fn create_move(&self) {
        if self.common() {
            debug!("moving from '{:?}' to '{:?}'", self.src, self.dst);
            fs::rename(&self.src, &self.dst).expect("move operation failed");
        }
    }

    fn common(&self) -> bool {
        if self.src.is_dir() {
            self.create_dst_dir();
            false
        } else {
            self.remove_dst();
            true
        }
    }
    fn create_dst_dir(&self) {
        if !self.dst.exists() {
            debug!("creating dir: {:?}", self.dst);
            fs::create_dir_all(&self.dst).expect("couldn't create dir");
        }
    }
    fn remove_dst(&self) {
        if self.dst.exists() {
            debug!("removing file '{:?}'", self.dst);
            fs::remove_file(&self.dst).expect("error in file deletion");
        }
    }
}

fn get_paths_after_replacing_args(
    job: &JobConfigs,
    args_map: &HashMap<String, String>,
) -> (String, String) {
    let mut src = job.src.clone();
    let mut dst = job.dst.clone();
    for (arg, val) in args_map {
        src = src.replace(arg, val.as_str());
        dst = dst.replace(arg, val);
    }
    (src, dst)
}

fn do_jobs(json_data: &AssetRelocationDef, args_map: &HashMap<String, String>) {
    for job in &json_data.jobs {
        let (src, mut dst) = get_paths_after_replacing_args(job, args_map);
        dst.push('\\');
        let src = fix_windows_path(src);
        let dst = fix_windows_path(dst);
        do_job(src.as_str(), dst.as_str(), job.todo.as_str());
    }
}

fn fix_windows_path(path: String) -> String {
    path.replace("/", "\\").replace("\\\\", "\\")
}

fn do_job(src: &str, dst: &str, todo: &str) {
    debug!("current job is: {:?}", todo);
    let paths = get_asset_paths_for_processing(src, dst);
    match todo {
        "hardlink" => paths.iter().for_each(|x| x.create_hardlink()),
        "copy" => paths.iter().for_each(|x| x.create_copy()),
        "move" => paths.iter().for_each(|x| x.create_move()),
        _ => panic!("this is new. not yet handled"),
    }
}

fn get_asset_paths_for_processing(src: &str, dst: &str) -> Vec<AssetPaths> {
    let mut paths: Vec<AssetPaths> = Vec::new();
    for item in WalkDir::new(Path::new(src)) {
        let item = item.unwrap();
        let src_path = &item.path();
        let src_str = src_path.to_str().unwrap();
        let src_delta = &src_str[src.len()..];
        let dst_str = dst.to_owned() + src_delta;
        let dst_str = dst_str.trim_end_matches("\\");
        let dst_path = Path::new(&dst_str);
        let apath = AssetPaths {
            src: src_path.to_path_buf(),
            dst: dst_path.to_path_buf(),
        };
        paths.push(apath);
    }
    paths
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
    setup_logger();
    log_panics::init();
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
    fn weird_paths() {
        assert_eq!(
            "c:\\abc\\der\\mea\\fal",
            fix_windows_path(String::from("c:\\/abc/der//mea\\fal")).as_str()
        );
    }
}
