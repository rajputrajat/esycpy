use log::{debug, trace};
use log_panics;
use regex::Regex;

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

#[derive(Debug)]
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
        trace!(
            "func: common(), file: {}, line: {}, src: {:?}, dst: {:?}",
            file!(),
            line!(),
            self.src,
            self.dst
        );
        if self.src.is_dir() {
            self.create_dst_dir();
            false
        } else {
            let dst_parent = self.dst.parent().unwrap();
            trace!("check dst parent: {:?}", dst_parent);
            if !dst_parent.exists() {
                fs::create_dir_all(dst_parent).expect("couldn't create parent dir");
            }
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
        trace!(
            "func: remove_dst, file: {}, line: {}, dst: {:?}",
            file!(),
            line!(),
            self.dst
        );
        if self.dst.exists() {
            debug!("removing file '{:?}'", self.dst);
            let meta_data = fs::metadata(&self.dst).unwrap();
            let mut permissions = meta_data.permissions();
            if permissions.readonly() {
                permissions.set_readonly(false);
                fs::set_permissions(&self.dst, permissions).expect("unable to set the file perm");
            }
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
    trace!("before processing: src: {}, dst: {}", src, dst);
    let mut paths: Vec<AssetPaths> = Vec::new();
    let mut src = src.trim_end_matches("\\").trim_end_matches("/");
    let re = Regex::new(r".+\\+\*\.([[:alnum:]]+)").unwrap();
    let mut filter = String::new();
    if let Some(captures) = re.captures(src) {
        if captures.len() == 2 {
            filter = captures[1].to_owned();
            let last_bslash = src.rfind("\\").unwrap();
            src = &src[..last_bslash];
        }
        //trace!("capture: {:?}", captures);
    }
    fn append_path(src_root: &str, dst_root: &str, src: &str) -> String {
        let src_delta = &src[src_root.len()..];
        trace!(
            "append_path. src_root: {:?}, dst_root: {:?}, src_delta: {:?}",
            src_root,
            dst_root,
            src_delta
        );
        let dst_str = dst_root.to_owned() + src_delta;
        dst_str.trim_end_matches("\\").to_owned()
    }
    fn is_hidden(entry: &Path) -> bool {
        entry
            .file_name()
            .map(|s| s.to_str().unwrap().starts_with("."))
            .unwrap_or(false)
    }
    if !filter.is_empty() {
        //trace!("filter value: {:?}, and src: {:?}", filter, src);
        for item in fs::read_dir(src).expect("problem") {
            let item = &item.unwrap().path();
            fn is_process_required(x: &Path, filter: &str) -> bool {
                //debug!("reached with: {:?}", x);
                if is_hidden(x) {
                    return false;
                } else if x.is_dir() {
                    return false;
                } else {
                    let file_extn = x.extension();
                    if let Some(v) = file_extn {
                        if v.to_str().unwrap().to_owned() == filter {
                            //debug!("ext found: {:?}", x);
                            return true;
                        }
                    }
                }
                false
            }
            if is_process_required(&item, filter.as_str()) {
                //debug!("src path after filter: {:?}", item);
                let src_path = item;
                let new_dst = append_path(src, dst, src_path.to_str().unwrap());
                let apath = AssetPaths {
                    src: src_path.to_path_buf(),
                    dst: PathBuf::from(new_dst),
                };
                paths.push(apath);
            }
        }
    } else {
        for item in WalkDir::new(Path::new(src)) {
            let item = item.unwrap();
            let src_path = item.path();
            let new_dst = append_path(src, dst, src_path.to_str().unwrap());
            let apath = AssetPaths {
                src: src_path.to_path_buf(),
                dst: PathBuf::from(new_dst),
            };
            paths.push(apath);
        }
    }
    //trace!("paths: {:#?}", paths);
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
        let json_data = parse_json(Path::new("esycpy_def.json"));
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

    #[test]
    fn test_regex() {
        let re = Regex::new(r"\*(\.[[:alnum:]]+)").unwrap();
        let m = re.captures("*.mercury").unwrap();
        assert_eq!(".mercury", &m[1]);
        let re = Regex::new(r".+\\+\*(\.[[:alnum:]]+)").unwrap();
        let m = re.captures("c:\\yo\\man\\*.mercury").unwrap();
        assert_eq!(".mercury", &m[1]);
        assert_eq!(m.len(), 2);
        let re = Regex::new(r".+\\+\*\.([[:alnum:]]+)").unwrap();
        let m = re.captures("c:\\yo\\man\\*.mercury").unwrap();
        assert_eq!("mercury", &m[1]);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_sting_find() {
        let test_str = "c:\\hello\\there\\*.xml";
        let ind = test_str.rfind("\\").unwrap();
        assert_eq!("c:\\hello\\there", &test_str[..ind]);
    }
}
