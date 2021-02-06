use crate::args::{ArgsType, Operation};
use std::fs;
use std::path::Path;

enum OperationTypes {
    FileToFile,
    DirToDir,
    AllFilesDirsToDir,
    AllSpecificFilesToDir,
    RecursiveAllSpecificFilesToDir
}

pub struct FileOp {
    op: Operation,
    from: String,
    to: String
}

impl FileOp {
    pub fn from(arg_paths: ArgsType) -> Self {
        match arg_paths {
            ArgsType::CmdLine{ op, from, to } => Self {
                op,
                from: FileOp::fix_path(from.as_str()),
                to: FileOp::fix_path(to.as_str())
            },
            _ => unreachable!(),
        }
    }

    pub fn process() {
    }

    fn file_op<P: AsRef<Path>>(&self, src: P, dst: P) {
        assert!(src.as_ref().exists());
        assert!(FileOp::is_dst_valid(dst.as_ref().to_str().unwrap()));
        if !dst.as_ref().parent().unwrap().exists() {
            fs::create_dir_all(dst.as_ref().parent().unwrap()).unwrap();
        }
        match self.op {
            Operation::Copy_ => {
                let _ = fs::copy(src.as_ref(), dst.as_ref())
                    .unwrap_or_else(|_| panic!("couldn't copy from {} to {}",
                            src.as_ref().to_str().unwrap(),
                            dst.as_ref().to_str().unwrap()));
            }
            Operation::Hardlink => {
                fs::hard_link(src.as_ref(), dst.as_ref())
                    .unwrap_or_else(|_| panic!("couldn't create hard_link, from {} to {}",
                            src.as_ref().to_str().unwrap(),
                            dst.as_ref().to_str().unwrap()));
            }
            Operation::Move => {
                fs::rename(src.as_ref(), dst.as_ref())
                    .unwrap_or_else(|_| panic!("couldn't move from {} to {}",
                            src.as_ref().to_str().unwrap(),
                            dst.as_ref().to_str().unwrap()));
            }
        }
    }

    fn file_to_file(&self) {

    }

    fn fix_path(input: &str) -> String {
        let forward_slash = input.replace("\\", "/");
        println!("{}", forward_slash.clone());
        let mut only_one_slash = String::new();
        let mut prev_char: Option<char> = None;
        forward_slash.chars().for_each(|c| {
            only_one_slash.push(c.clone());
            if c == '/' && prev_char == Some('/') {
                only_one_slash.pop();
            }
            prev_char = Some(c);
        });
        only_one_slash
    }

    fn is_dst_valid(dst: &str) -> bool {
        let mut dst_path = Path::new(dst);
        loop {
            if dst_path.exists() {
                return true;
            } else {
                let parent = dst_path.parent();
                if let Some(parent) = parent {
                    dst_path = parent;
                } else {
                    return false;
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    #[should_panic]
    fn check_ops_hardlink_file_err() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let file_op = FileOp { op: Operation::Hardlink, from: String::new(), to: String::new() };
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        file_op.file_op(
            src_file.to_str().unwrap().to_owned(),
            dst_file.to_str().unwrap().to_owned()
        );
        assert!(src_file.exists());
        assert!(dst_file.exists());
        let src_file_text = fs::read_to_string(src_file).unwrap();
        let dst_file_text = fs::read_to_string(dst_file).unwrap();
        assert_eq!(src_file_text, dst_file_text);
    }

    #[test]
    fn check_ops_hardlink_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let _ = fs::copy("test_files/for_file_operations/sample_file", src_file.as_path())
            .unwrap();
        assert!(src_file.exists());
        let file_op = FileOp { op: Operation::Hardlink, from: String::new(), to: String::new() };
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        file_op.file_op(
            src_file.to_str().unwrap().to_owned(),
            dst_file.to_str().unwrap().to_owned()
        );
        assert!(src_file.exists());
        assert!(dst_file.exists());
        let src_file_text = fs::read_to_string(src_file).unwrap();
        let dst_file_text = fs::read_to_string(dst_file).unwrap();
        assert_eq!(src_file_text, dst_file_text);
    }

    #[test]
    fn check_ops_move_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let _ = fs::copy("test_files/for_file_operations/sample_file", src_file.as_path())
            .unwrap();
        assert!(src_file.exists());
        let file_op = FileOp { op: Operation::Move, from: String::new(), to: String::new() };
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        file_op.file_op(
            src_file.to_str().unwrap().to_owned(),
            dst_file.to_str().unwrap().to_owned()
        );
        assert!(!src_file.exists());
        assert!(dst_file.exists());
        let src_file_text = fs::read_to_string("test_files/for_file_operations/sample_file")
            .unwrap();
        let dst_file_text = fs::read_to_string(dst_file).unwrap();
        assert_eq!(src_file_text, dst_file_text);
    }

    #[test]
    fn check_ops_copy_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let _ = fs::copy("test_files/for_file_operations/sample_file", src_file.as_path())
            .unwrap();
        assert!(src_file.exists());
        let file_op = FileOp { op: Operation::Copy_, from: String::new(), to: String::new() };
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        file_op.file_op(
            src_file.to_str().unwrap().to_owned(),
            dst_file.to_str().unwrap().to_owned()
        );
        assert!(src_file.exists());
        assert!(dst_file.exists());
        let src_file_text = fs::read_to_string(src_file).unwrap();
        let dst_file_text = fs::read_to_string(dst_file).unwrap();
        assert_eq!(src_file_text, dst_file_text);
    }

    #[test]
    fn fix_path_positive() {
        assert_eq!(
            FileOp::fix_path("c:\\\\\\Users\\\\\\\\test///dir"),
            String::from("c:/Users/test/dir"));
        assert_eq!(FileOp::fix_path("/mnt///dr"), String::from("/mnt/dr"));
    }

    #[test]
    fn dst_valid() {
        assert_eq!(FileOp::is_dst_valid("c:/users/test/invalid_path"), true);
        assert_eq!(FileOp::is_dst_valid(" c:/users/test/invalid_path"), false);
        assert_eq!(FileOp::is_dst_valid(" \\Debug\\bin"), false);
        assert_eq!(FileOp::is_dst_valid("c:\\users\\Debug\\bin"), true);
    }
}
