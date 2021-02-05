use crate::args::{ArgsType, Operation};
use std::fs;

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

    #[test]
    fn fix_path_positive() {
        assert_eq!(FileOp::fix_path("c:\\\\\\Users\\\\\\\\test///dir"), String::from("c:/Users/test/dir"));
        assert_eq!(FileOp::fix_path("/mnt///dr"), String::from("/mnt/dr"));
    }
}
