use log::debug;
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
    fn create_hardlink(&self) -> Option<String>;
    fn create_copy(&self) -> Option<String> {
        None
    }
    fn create_move(&self) -> Option<String> {
        None
    }
    fn remove_dst(&self) -> Option<String> {
        None
    }
    fn create_dir(&self, _: &Path) -> Option<String> {
        None
    }
    fn recurse(&self, _: fn(&Self) -> Option<String>) -> Option<String> {
        None
    }
}

struct AssetPaths {
    src: PathBuf,
    dst: PathBuf,
}

impl FileOperations for AssetPaths {
    fn recurse(&self, cb: fn(&Self) -> Option<String>) -> Option<String> {
        for item in WalkDir::new(&self.src) {
            let item = item.unwrap();
            let src_path = &item.path();
            let src_str = src_path.to_str().unwrap();
            let src_delta = &src_str[self.src.to_str().unwrap().len()..];
            let dst_str = self.dst.to_str().unwrap().to_owned() + src_delta;
            let dst_path = Path::new(&dst_str);
            if src_path.is_dir() {
                self.create_dir(dst_path)?;
            } else {
                cb(&self)?;
            }
        }
        None
    }
    fn create_dir(&self, d: &Path) -> Option<String> {
        if !d.exists() {
            debug!("creating dir: {:?}", d);
            fs::create_dir(d).expect("couldn't create dir");
        }
        None
    }

    fn create_hardlink(&self) -> Option<String> {
        debug!(
            "creating hardlink from '{:?}' to '{:?}'",
            self.src, self.dst
        );
        fs::hard_link(&self.src, &self.dst).expect("couldn't create hardlink");
        None
    }

    fn create_copy(&self) -> Option<String> {
        debug!("copying from '{:?}' to '{:?}'", self.src, self.dst);
        let _ = fs::copy(&self.src, &self.dst).expect("problem while copying");
        None
    }

    fn create_move(&self) -> Option<String> {
        debug!("moving from '{:?}' to '{:?}'", self.src, self.dst);
        fs::rename(&self.src, &self.dst).expect("move operation failed");
        None
    }

    fn remove_dst(&self) -> Option<String> {
        if self.dst.exists() {
            debug!("removing file '{:?}'", self.dst);
            fs::remove_file(&self.dst).expect("error in file deletion");
        }
        None
    }
}

fn get_paths_after_replacing_args(
    job: &JobConfigs,
    args_map: &HashMap<String, String>,
) -> (PathBuf, PathBuf) {
    let mut src = job.src.clone();
    let mut dst = job.dst.clone();
    for (arg, val) in args_map {
        src = src.replace(arg, val.as_str());
        dst = dst.replace(arg, val);
    }
    (PathBuf::from(src.as_str()), PathBuf::from(dst.as_str()))
}

fn do_jobs(json_data: &AssetRelocationDef, args_map: &HashMap<String, String>) {
    for job in &json_data.jobs {
        let (src, dst) = get_paths_after_replacing_args(job, args_map);
        let asset_paths = AssetPaths { src, dst };
        let _ = do_job(job, &asset_paths);
    }
}

fn do_job(job: &JobConfigs, asset_paths: &impl FileOperations) -> Option<String> {
    debug!("current job is: {:?}", job);
    asset_paths.remove_dst();
    match job.todo.as_str() {
        "hardlink" => asset_paths.recurse(FileOperations::create_hardlink),
        "copy" => asset_paths.recurse(FileOperations::create_copy),
        "move" => asset_paths.recurse(FileOperations::create_move),
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
    use std::fmt::Write;
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

    struct TestAssetPaths {
        src: PathBuf,
        dst: PathBuf,
    }
    impl FileOperations for TestAssetPaths {
        fn create_hardlink(&self) -> Option<String> {
            let mut ret = String::new();
            write!(
                &mut ret,
                "hardlink, {}, {}",
                self.src.to_str().unwrap(),
                self.dst.to_str().unwrap()
            )
            .expect("?");
            Some(ret)
        }
        fn create_copy(&self) -> Option<String> {
            let mut ret = String::new();
            write!(
                &mut ret,
                "copy, {}, {}",
                self.src.to_str().unwrap(),
                self.dst.to_str().unwrap()
            )
            .expect("?");
            Some(ret)
        }
    }

    #[test]
    fn which_operation() {
        let args = "one, two, three, four, five, six, seven";
        let json_data = parse_json(Path::new("asset_relocation_def.json"));
        let args_map = map_args(&json_data, String::from(args));
        {
            let job = &json_data.jobs[0];
            let (src, dst) = get_paths_after_replacing_args(job, &args_map);
            let test_asset_paths = TestAssetPaths { src, dst };
            assert_eq!(
                "hardlink, four/assets/setup.txt, six/setup.txt",
                do_job(job, &test_asset_paths).unwrap()
            );
        }
        {
            let job = &json_data.jobs[4];
            let (src, dst) = get_paths_after_replacing_args(job, &args_map);
            let test_asset_paths = TestAssetPaths { src, dst };
            assert_eq!(
                concat!(
                    "hardlink, ",
                    "six/../../../../Tools/GDKRuntimeHost/two/one, ",
                    "seven/one/Runtime/bin"
                ),
                do_job(job, &test_asset_paths).unwrap()
            );
        }
        {
            let job = &json_data.jobs[7];
            let (src, dst) = get_paths_after_replacing_args(job, &args_map);
            let test_asset_paths = TestAssetPaths { src, dst };
            assert_eq!("copy, , ", do_job(job, &test_asset_paths).unwrap());
        }
    }
}
