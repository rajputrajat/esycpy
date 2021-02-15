use crate::args::{ArgsType, Operation};
use anyhow::Result;
use log::trace;
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
            ..Default::default()
        }
    }
}

impl FileOp {
    fn update_src_and_file_type(&mut self) {}

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
        Ok(())
    }

    fn file_to_file(&self) -> Result<()> {
        self.file_op(&vec![self.p.clone()])?;
        Ok(())
    }

    fn fix_offset(p: &Paths, new_src: &str) -> String {
        let offset = new_src.replace(&p.from, "");
        let mut dst = String::from(&p.to);
        dst.push_str(&offset);
        dst
    }

    fn get_src_dst_paths<F>(&self, fname_filter: F, only_cur_dir: bool) -> Result<Vec<Paths>>
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
            let dst = FileOp::fix_offset(&self.p, src.to_str().unwrap());
            paths.push(Paths {
                from: src.to_str().unwrap().to_owned(),
                to: dst,
            })
        }
        Ok(paths)
    }

    fn dir_to_dir(&self) -> Result<()> {
        match self.op {
            Some(Operation::Copy_) => {
                let copy_options = fs_extra::dir::CopyOptions {
                    overwrite: true,
                    ..Default::default()
                };
                fs_extra::dir::copy(&self.p.from, &self.p.to, &copy_options)?;
            }
            Some(Operation::Move) => self.file_op(&vec![self.p.clone()])?,
            Some(Operation::Hardlink) => {}
            None => unreachable!(),
        }
        Ok(())
    }
    fn all_files_dirs_to_dir(&self) -> Result<()> {
        let copy_options = fs_extra::dir::CopyOptions {
            overwrite: true,
            content_only: true,
            ..Default::default()
        };
        fs_extra::dir::copy(&self.p.from, &self.p.to, &copy_options)?;
        Ok(())
    }
    fn all_specific_files_to_dir(&self) -> Result<()> {
        Ok(())
    }
    fn recursive_all_specific_files_to_dir(&self) -> Result<()> {
        Ok(())
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

    #[test]
    #[should_panic]
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    //#[ignore]
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
        let mut v_returned = file_op.get_src_dst_paths(|_| true, false).unwrap();
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
    fn check_paths_of_only_cur_dir() {
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
                    let ext = f.file_name().to_str().unwrap().rsplit(|c| c == '.').next().unwrap();
                    ext == "file"
                },
                false,
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
                to: format!("{}\\d1\\f11.file", s_dst),
            },
            Paths {
                from: format!("{}\\d1\\d12\\f12.file", parent(&s_src)),
                to: format!("{}\\d1\\d12\\f12.file", s_dst),
            },
        ];
        fix_path_vec(&mut v_test);
        v_test.sort_unstable();
        assert_eq!(v_returned, v_test);
    }

    fn fix_path_vec(v: &mut Vec<Paths>) {
        for p in v {
            p.from = FileOp::fix_path(&p.from);
            p.to = FileOp::fix_path(&p.to);
        }
    }
}
