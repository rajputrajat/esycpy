use crate::args::{ArgsType, Operation};
use anyhow::Result;
use log::trace;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct FileOp {
    op: Option<Operation>,
    p: Paths,
    f_type: Option<FileType>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Paths {
    from: String,
    to: String,
}

#[derive(Debug, PartialEq)]
enum FileType {
    File,
    Dir,
    Filter(String),
}

impl Default for FileOp {
    fn default() -> Self {
        Self {
            op: None,
            f_type: None,
            p: Paths::default(),
        }
    }
}

impl FileOp {
    pub fn from(arg_paths: ArgsType) -> Self {
        let file_op = match arg_paths {
            ArgsType::CmdLine { op, from, to } => {
                let mut from = FileOp::fix_path(&from);
                let to = FileOp::fix_path(&to);
                let file_path = Path::new(&from);
                let file_name = file_path
                    .file_name()
                    .expect("file name must be present")
                    .to_str()
                    .unwrap();
                let f_type: Option<FileType>;
                if file_name.contains('*') {
                    f_type = Some(FileType::Filter(file_name.to_owned()));
                    from = file_path.parent().unwrap().to_str().unwrap().to_owned();
                } else {
                    if file_path.is_dir() {
                        f_type = Some(FileType::Dir);
                    } else {
                        f_type = Some(FileType::File);
                    }
                }
                Self {
                    op: Some(op),
                    p: Paths { from, to },
                    f_type,
                }
            }
            _ => unreachable!(),
        };
        file_op
    }

    pub fn process(&self) -> Result<()> {
        trace!("processing {:?}", self);
        match &self.f_type {
            Some(FileType::File) => self.file_op(&vec![self.p.clone()])?,
            Some(FileType::Dir) => self.dir_to_dir()?,
            Some(FileType::Filter(file_name)) => {
                let mut only_cur_dir = true;
                if file_name.contains("**") {
                    only_cur_dir = false;
                }
                let ext = FileOp::get_ext(file_name)?;
                let filter = |f: &DirEntry| -> bool {
                    if ext == "" {
                        true
                    } else if f.file_name().to_str().unwrap().contains(&ext) {
                        true
                    } else {
                        false
                    }
                };
                let v_paths = self.get_src_dst_paths(filter, only_cur_dir, !ext.is_empty())?;
                self.file_op(&v_paths)?;
            }
            None => unreachable!(),
        }
        Ok(())
    }

    fn get_ext(file_name: &str) -> Result<String> {
        let re = Regex::new(r"\**(.*)$")?;
        let cap = re.captures(file_name).unwrap();
        Ok(cap[1].to_owned())
    }

    fn dir_to_dir(&self) -> Result<()> {
        match self.op {
            Some(Operation::Move) => self.file_op(&vec![self.p.clone()])?,
            Some(Operation::Hardlink) | Some(Operation::Copy_) => {
                let v_paths = self.get_src_dst_paths(|f| f.path().is_file(), false, false)?;
                self.file_op(&v_paths)?;
            }
            None => unreachable!(),
        }
        Ok(())
    }

    fn fix_offset(p: &Paths, new_src: &str) -> String {
        let offset = new_src.replace(&p.from, "");
        let mut dst = String::from(&p.to);
        dst.push_str(&offset);
        dst
    }

    fn get_src_dst_paths<F>(&self, fname_filter: F, only_cur_dir: bool, ext_specified: bool)
        -> Result<Vec<Paths>>
    where
        F: Fn(&DirEntry) -> bool,
    {
        let mut paths: Vec<Paths> = Vec::new();
        let mut dir_walker = WalkDir::new(&self.p.from);
        if only_cur_dir {
            dir_walker = dir_walker.max_depth(1);
        }
        for file in dir_walker
            .into_iter()
            .filter(|f| f.as_ref().unwrap().path().is_file())
            .filter(|f| fname_filter(f.as_ref().unwrap()))
        {
            let file = file.unwrap();
            let src = file.path();
            let dst: String;
            if ext_specified {
                dst = Path::new(&self.p.to).join(file.file_name()).to_str().unwrap().to_owned();
            } else {
                dst = FileOp::fix_offset(&self.p, src.to_str().unwrap());
            }
            paths.push(Paths {
                from: src.to_str().unwrap().to_owned(),
                to: dst,
            })
        }
        println!("{:#?}", paths);
        trace!("{:?}", paths);
        Ok(paths)
    }

    fn file_op(&self, vp: &Vec<Paths>) -> Result<()> {
        for p in vp {
            trace!("{:?}", p);
            let src = Path::new(&p.from);
            let dst = Path::new(&p.to);
            assert!(src.exists());
            assert!(FileOp::is_dst_valid(dst.to_str().unwrap()));
            if !dst.parent().unwrap().exists() {
                fs::create_dir_all(dst.parent().unwrap())?;
            }
            match self.op {
                Some(Operation::Copy_) => {
                    let _ = fs::copy(&src, &dst)?;
                }
                Some(Operation::Hardlink) => fs::hard_link(&src, &dst)?,
                Some(Operation::Move) => fs::rename(&src, &dst)?,
                None => unreachable!(),
            }
        }
        Ok(())
    }

    fn fix_path(input: &str) -> String {
        let forward_slash = input.replace("\\", "/");
        trace!("{}", forward_slash.clone());
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
    use anyhow::anyhow;

    #[test]
    #[should_panic]
    fn check_ops_hardlink_file_err() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let file_op = FileOp {
            op: Some(Operation::Hardlink),
            p: Paths {
                from: String::new(),
                to: String::new(),
            },
            ..Default::default()
        };
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        file_op
            .file_op(&vec![Paths {
                from: src_file.to_str().unwrap().to_owned(),
                to: dst_file.to_str().unwrap().to_owned(),
            }])
            .unwrap();
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
        let _ = fs::copy(
            "test_files/for_file_operations/sample_file",
            src_file.as_path(),
        )
        .unwrap();
        assert!(src_file.exists());
        let file_op = FileOp {
            op: Some(Operation::Hardlink),
            p: Paths {
                from: String::new(),
                to: String::new(),
            },
            ..Default::default()
        };
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        file_op
            .file_op(&vec![Paths {
                from: src_file.to_str().unwrap().to_owned(),
                to: dst_file.to_str().unwrap().to_owned(),
            }])
            .unwrap();
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
        let _ = fs::copy(
            "test_files/for_file_operations/sample_file",
            src_file.as_path(),
        )
        .unwrap();
        assert!(src_file.exists());
        let file_op = FileOp {
            op: Some(Operation::Move),
            p: Paths {
                from: String::new(),
                to: String::new(),
            },
            ..Default::default()
        };
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        file_op
            .file_op(&vec![Paths {
                from: src_file.to_str().unwrap().to_owned(),
                to: dst_file.to_str().unwrap().to_owned(),
            }])
            .unwrap();
        assert!(!src_file.exists());
        assert!(dst_file.exists());
        let src_file_text =
            fs::read_to_string("test_files/for_file_operations/sample_file").unwrap();
        let dst_file_text = fs::read_to_string(dst_file).unwrap();
        assert_eq!(src_file_text, dst_file_text);
    }

    #[test]
    fn check_ops_copy_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let _ = fs::copy(
            "test_files/for_file_operations/sample_file",
            src_file.as_path(),
        )
        .unwrap();
        assert!(src_file.exists());
        let file_op = FileOp {
            op: Some(Operation::Copy_),
            p: Paths {
                from: String::new(),
                to: String::new(),
            },
            ..Default::default()
        };
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        file_op
            .file_op(&vec![Paths {
                from: src_file.to_str().unwrap().to_owned(),
                to: dst_file.to_str().unwrap().to_owned(),
            }])
            .unwrap();
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
            String::from("c:/Users/test/dir")
        );
        assert_eq!(FileOp::fix_path("/mnt///dr"), String::from("/mnt/dr"));
    }

    #[test]
    fn dst_valid() {
        assert_eq!(FileOp::is_dst_valid("c:/users/test/invalid_path"), true);
        assert_eq!(FileOp::is_dst_valid(" c:/users/test/invalid_path"), false);
        assert_eq!(FileOp::is_dst_valid(" \\Debug\\bin"), false);
        assert_eq!(FileOp::is_dst_valid("c:\\users\\Debug\\bin"), true);
    }

    #[test]
    fn check_fix_offset() {
        assert_eq!(
            FileOp::fix_offset(
                &Paths {
                    from: "c:/Users/test/dir1".to_owned(),
                    to: "c:/Users/test/dir2".to_owned()
                },
                "c:/Users/test/dir1/dir3/dir4/a_file"
            ),
            "c:/Users/test/dir2/dir3/dir4/a_file"
        );
    }

    #[test]
    fn check_get_src_dst_paths() {
        println!("reached here");
        let tmp_dir = TempDir::new().unwrap();
        let dst_dir = tmp_dir.path().join("dst");
        let s_src = "./test_files/test_src_dst_paths".to_owned();
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        println!("reached here");
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Move,
            from: s_src.clone(),
            to: s_dst.clone(),
        });
        println!("{:?}", file_op);
        let mut v_returned = file_op.get_src_dst_paths(|_| true, false, false).unwrap();
        fix_path_vec(&mut v_returned);
        v_returned.sort_unstable();
        let mut v_test: Vec<Paths> = vec![
            Paths {
                from: format!("{}\\f1.file", s_src),
                to: format!("{}\\f1.file", s_dst),
            },
            Paths {
                from: format!("{}\\d1\\f11.file", s_src),
                to: format!("{}\\d1\\f11.file", s_dst),
            },
            Paths {
                from: format!("{}\\d1\\d12\\f12.file", s_src),
                to: format!("{}\\d1\\d12\\f12.file", s_dst),
            },
            Paths {
                from: format!("{}\\d3\\f3.img", s_src),
                to: format!("{}\\d3\\f3.img", s_dst),
            },
        ];
        fix_path_vec(&mut v_test);
        v_test.sort_unstable();
        assert_eq!(v_returned, v_test);
    }

    #[test]
    fn check_paths_specific_file_only_cur_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let dst_dir = tmp_dir.path().join("dst");
        let s_src = "./test_files/test_src_dst_paths/*.file".to_owned();
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Move,
            from: s_src.clone(),
            to: s_dst.clone(),
        });
        println!("{:?}", file_op);
        let mut v_returned = file_op
            .get_src_dst_paths(
                |f| {
                    let ext = f
                        .file_name()
                        .to_str()
                        .unwrap()
                        .rsplit(|c| c == '.')
                        .next()
                        .unwrap();
                    ext == "file"
                },
                true,
                true
            )
            .unwrap();
        fix_path_vec(&mut v_returned);
        v_returned.sort_unstable();
        let parent = |s: &str| Path::new(s).parent().unwrap().to_str().unwrap().to_owned();
        let mut v_test: Vec<Paths> = vec![Paths {
            from: format!("{}\\f1.file", parent(&s_src)),
            to: format!("{}\\f1.file", s_dst),
        }];
        fix_path_vec(&mut v_test);
        v_test.sort_unstable();
        assert_eq!(v_returned, v_test);
    }

    #[test]
    fn check_paths_recursive_specific_file_type() {
        let tmp_dir = TempDir::new().unwrap();
        let dst_dir = tmp_dir.path().join("dst");
        let s_src = "./test_files/test_src_dst_paths/**.file".to_owned();
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Move,
            from: s_src.clone(),
            to: s_dst.clone(),
        });
        println!("{:?}", file_op);
        let mut v_returned = file_op
            .get_src_dst_paths(
                |f| {
                    let ext = f
                        .file_name()
                        .to_str()
                        .unwrap()
                        .rsplit(|c| c == '.')
                        .next()
                        .unwrap();
                    ext == "file"
                },
                false,
                true
            )
            .unwrap();
        fix_path_vec(&mut v_returned);
        v_returned.sort_unstable();
        let parent = |s: &str| Path::new(s).parent().unwrap().to_str().unwrap().to_owned();
        let mut v_test: Vec<Paths> = vec![
            Paths {
                from: format!("{}\\f1.file", parent(&s_src)),
                to: format!("{}\\f1.file", s_dst),
            },
            Paths {
                from: format!("{}\\d1\\f11.file", parent(&s_src)),
                to: format!("{}\\f11.file", s_dst),
            },
            Paths {
                from: format!("{}\\d1\\d12\\f12.file", parent(&s_src)),
                to: format!("{}\\f12.file", s_dst),
            },
        ];
        fix_path_vec(&mut v_test);
        v_test.sort_unstable();
        assert_eq!(v_returned, v_test);
    }

    #[test]
    fn get_ext_() {
        assert_eq!(FileOp::get_ext("*").unwrap(), "");
        assert_eq!(FileOp::get_ext("*.txt").unwrap(), ".txt");
        assert_eq!(FileOp::get_ext("**.file").unwrap(), ".file");
        assert_eq!(FileOp::get_ext("**").unwrap(), "");
        assert_eq!(FileOp::get_ext("**suffix").unwrap(), "suffix");
    }

    #[test]
    fn files_from_only_this_dir() {}

    #[test]
    fn copy_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let _ = fs::copy(
            "test_files/for_file_operations/sample_file",
            src_file.as_path(),
        )
        .unwrap();
        assert!(src_file.exists());
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Copy_,
            from: src_file.to_str().unwrap().to_owned(),
            to: dst_file.to_str().unwrap().to_owned(),
        });
        file_op.process().unwrap();
        assert!(src_file.exists());
        assert!(dst_file.exists());
        let src_file_text = fs::read_to_string(src_file).unwrap();
        let dst_file_text = fs::read_to_string(dst_file).unwrap();
        assert_eq!(src_file_text, dst_file_text);
    }

    fn fix_path_vec(v: &mut Vec<Paths>) {
        for p in v {
            p.from = FileOp::fix_path(&p.from);
            p.to = FileOp::fix_path(&p.to);
        }
    }

    #[test]
    fn move_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let _ = fs::copy(
            "test_files/for_file_operations/sample_file",
            src_file.as_path(),
        )
        .unwrap();
        assert!(src_file.exists());
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Move,
            from: src_file.to_str().unwrap().to_owned(),
            to: dst_file.to_str().unwrap().to_owned(),
        });
        file_op.process().unwrap();
        assert!(!src_file.exists());
        assert!(dst_file.exists());
    }

    #[test]
    fn hardlink_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir_all(src_dir.as_path()).unwrap();
        let src_file = src_dir.join("sample_file");
        let _ = fs::copy(
            "test_files/for_file_operations/sample_file",
            src_file.as_path(),
        )
        .unwrap();
        assert!(src_file.exists());
        let dst_dir = tmp_dir.path().join("dst");
        let dst_file = dst_dir.join("sample_file");
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Hardlink,
            from: src_file.to_str().unwrap().to_owned(),
            to: dst_file.to_str().unwrap().to_owned(),
        });
        file_op.process().unwrap();
        assert!(src_file.exists());
        assert!(dst_file.exists());
        let src_file_text = fs::read_to_string(src_file).unwrap();
        let dst_file_text = fs::read_to_string(dst_file).unwrap();
        assert_eq!(src_file_text, dst_file_text);
    }

    #[test]
    fn copy_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let dst_dir = tmp_dir.path().join("dst");
        let base = Path::new("./test_files/test_src_dst_paths");
        let src = tmp_dir.path().join("src");
        fs::create_dir_all(src.clone()).unwrap();
        fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default()).unwrap();
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Copy_,
            from: src.to_str().unwrap().to_owned(),
            to: s_dst.clone(),
        });
        file_op.process().unwrap();
        let v_src: Vec<String> = WalkDir::new(src)
            .into_iter()
            .map(|f| f.unwrap().path().to_str().unwrap().to_owned())
            .collect();
        let _v_dst = WalkDir::new(s_dst)
            .into_iter()
            .for_each(|f|
        {
            let dst = f.unwrap().path().to_str().unwrap().to_owned();
            assert!(v_src.iter().any(|s| s == &dst.replace("\\dst", "\\src")));
        });
    }

    #[test]
    fn move_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let dst_dir = tmp_dir.path().join("dst");
        let base = Path::new("./test_files/test_src_dst_paths");
        let src = tmp_dir.path().join("src");
        fs::create_dir_all(src.clone()).unwrap();
        fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default()).unwrap();
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Move,
            from: src.to_str().unwrap().to_owned(),
            to: s_dst.clone(),
        });
        let v_src: Vec<String> = WalkDir::new(&src)
            .into_iter()
            .map(|f| f.unwrap().path().to_str().unwrap().to_owned())
            .collect();
        file_op.process().unwrap();
        let _v_dst = WalkDir::new(s_dst)
            .into_iter()
            .for_each(|f|
        {
            let dst = f.unwrap().path().to_str().unwrap().to_owned();
            println!("{}", dst);
            assert!(v_src.iter().any(|s| s == &dst.replace("\\dst", "\\src")));
        });
        assert!(!src.exists());
    }

    #[test]
    fn hardlink_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let dst_dir = tmp_dir.path().join("dst");
        let base = Path::new("./test_files/test_src_dst_paths");
        let src = tmp_dir.path().join("src");
        fs::create_dir_all(src.clone()).unwrap();
        fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default()).unwrap();
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Hardlink,
            from: src.to_str().unwrap().to_owned(),
            to: s_dst.clone(),
        });
        file_op.process().unwrap();
        let v_src: Vec<String> = WalkDir::new(src)
            .into_iter()
            .map(|f| f.unwrap().path().to_str().unwrap().to_owned())
            .collect();
        let _v_dst = WalkDir::new(s_dst)
            .into_iter()
            .for_each(|f|
        {
            let dst = f.unwrap().path().to_str().unwrap().to_owned();
            assert!(v_src.iter().any(|s| s == &dst.replace("\\dst", "\\src")));
        });
    }

    #[test]
    fn copy_specific_files_cur_dir() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let dst_dir = tmp_dir.path().join("dst");
        let base = Path::new("./test_files/test_src_dst_paths");
        let src = tmp_dir.path().join("src");
        fs::create_dir_all(src.clone())?;
        fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default())?;
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Copy_,
            from: Path::new(&src)
                .join("test_src_dst_paths").join("*.file").to_str().unwrap().to_owned(),
            to: s_dst.clone(),
        });
        file_op.process()?;
        assert!(Path::new(&s_dst).join("f1.file").exists());
        assert!(!Path::new(&s_dst).join("d1").join("d12").join("f12.file").exists());
        Ok(())
    }

    #[test]
    fn copy_specific_files_recursively() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let dst_dir = tmp_dir.path().join("dst");
        let base = Path::new("./test_files/test_src_dst_paths");
        let src = tmp_dir.path().join("src");
        fs::create_dir_all(src.clone())?;
        fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default())?;
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Copy_,
            from: Path::new(&src)
                .join("test_src_dst_paths").join("**.file").to_str().unwrap().to_owned(),
            to: s_dst.clone(),
        });
        file_op.process()?;
        assert!(Path::new(&s_dst).join("f1.file").exists());
        assert!(Path::new(&s_dst).join("f12.file").exists());
        assert!(Path::new(&s_dst).join("f11.file").exists());
        assert!(!Path::new(&s_dst).join("f3.img").exists());
        Ok(())
    }

    #[test]
    fn copy_specific_files_recursively_2() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let dst_dir = tmp_dir.path().join("dst");
        let base = Path::new("./test_files/test_src_dst_paths");
        let src = tmp_dir.path().join("src");
        fs::create_dir_all(src.clone())?;
        fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default())?;
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Copy_,
            from: Path::new(&src)
                .join("test_src_dst_paths").join("**.img").to_str().unwrap().to_owned(),
            to: s_dst.clone(),
        });
        file_op.process()?;
        assert!(!Path::new(&s_dst).join("f1.file").exists());
        assert!(!Path::new(&s_dst).join("f12.file").exists());
        assert!(!Path::new(&s_dst).join("f11.file").exists());
        assert!(Path::new(&s_dst).join("f3.img").exists());
        Ok(())
    }

    #[test]
    fn copy_all_files_recursively() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let dst_dir = tmp_dir.path().join("dst");
        let base = Path::new("./test_files/test_src_dst_paths");
        let src = tmp_dir.path().join("src");
        fs::create_dir_all(src.clone())?;
        fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default())?;
        let s_dst = dst_dir.to_str().unwrap().to_owned();
        let file_op = FileOp::from(ArgsType::CmdLine {
            op: Operation::Copy_,
            from: Path::new(&src)
                .join("test_src_dst_paths").join("**").to_str().unwrap().to_owned(),
            to: s_dst.clone(),
        });
        file_op.process()?;
        assert!(Path::new(&s_dst).join("f1.file").exists());
        assert!(Path::new(&s_dst).join("d1").join("d12").join("f12.file").exists());
        assert!(Path::new(&s_dst).join("d1").join("f11.file").exists());
        assert!(Path::new(&s_dst).join("d3").join("f3.img").exists());
        Ok(())
    }
}
