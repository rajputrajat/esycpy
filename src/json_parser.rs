use crate::args::{ArgsType, Operation};
use log::debug;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_json_args(args: ArgsType) -> Vec<ArgsType> {
    match args {
        ArgsType::Json {
            json_file,
            variables,
        } => {
            let json_def = parse_json(Path::new(&json_file));
            map_variables(json_def, variables)
        }
        _ => unreachable!(),
    }
}

fn map_variables(
    asset_def: AssetRelocationDef,
    variables: Option<Vec<(String, String)>>,
) -> Vec<ArgsType> {
    assert_eq!(
        variables.is_none(),
        asset_def.variables_in_use.is_empty(),
        "either json file is missing vars, or command line"
    );
    if variables.is_some() {
        assert_eq!(
            variables.clone().unwrap().len(),
            asset_def.variables_in_use.len(),
            "vars count mismatch"
        );
    }
    let mut mapped_args: Vec<ArgsType> = Vec::new();
    asset_def.jobs.into_iter().for_each(|mut d| {
        let todo: Operation = match d.todo.as_str() {
            "copy" => Operation::Copy_,
            "move" => Operation::Move,
            "hardlink" => Operation::Hardlink,
            _ => panic!("unhandled operation"),
        };
        if let Some(variables) = variables.clone() {
            variables.iter().for_each(|v| {
                d.src = d
                    .src
                    .replace(&format!("{{{}}}", v.0.as_str()), v.1.as_str());
                d.dst = d
                    .dst
                    .replace(&format!("{{{}}}", v.0.as_str()), v.1.as_str());
            });
        };
        let mapped_arg = ArgsType::CmdLine {
            op: todo,
            from: PathBuf::from(d.src),
            to: PathBuf::from(d.dst),
        };
        mapped_args.push(mapped_arg)
    });
    mapped_args
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn map_var_correct() {
        let asset_def = AssetRelocationDef {
            variables_in_use: vec![
                "var1".to_owned(),
                "var2".to_owned(),
                "var3".to_owned(),
                "var4".to_owned(),
            ],
            jobs: vec![
                JobConfigs {
                    todo: "copy".to_owned(),
                    src: "this/is/{var1}/yes".to_owned(),
                    dst: "this/is/{var2}/yes".to_owned(),
                },
                JobConfigs {
                    todo: "hardlink".to_owned(),
                    src: "this/is/{var4}/yes".to_owned(),
                    dst: "this/is/{var3}/yes".to_owned(),
                },
                JobConfigs {
                    todo: "move".to_owned(),
                    src: "this/is/{var2}/yes".to_owned(),
                    dst: "this/is/{var3}/yes".to_owned(),
                },
            ],
        };
        let variables = Some(vec![
            (String::from("var1"), String::from("VAR1")),
            (String::from("var2"), String::from("VAR2")),
            (String::from("var3"), String::from("VAR3")),
            (String::from("var4"), String::from("VAR4")),
        ]);
        let mut ops = vec![Operation::Move, Operation::Hardlink, Operation::Copy_];
        let mut arg_types: Vec<ArgsType> = Vec::new();
        asset_def.jobs.iter().for_each(|d| {
            arg_types.push(ArgsType::CmdLine {
                op: ops.pop().unwrap(),
                to: PathBuf::from(
                    d.dst
                        .replace("var", "VAR")
                        .replace("{", "")
                        .replace("}", ""),
                ),
                from: PathBuf::from(
                    d.src
                        .replace("var", "VAR")
                        .replace("{", "")
                        .replace("}", ""),
                ),
            })
        });
        assert_eq!(arg_types, map_variables(asset_def, variables));
    }

    #[test]
    fn no_vars() {
        let asset_def = AssetRelocationDef {
            variables_in_use: vec![],
            jobs: vec![
                JobConfigs {
                    todo: "copy".to_owned(),
                    src: "this/is/var1/yes".to_owned(),
                    dst: "this/is/var2/yes".to_owned(),
                },
                JobConfigs {
                    todo: "hardlink".to_owned(),
                    src: "this/is/var4/yes".to_owned(),
                    dst: "this/is/var3/yes".to_owned(),
                },
                JobConfigs {
                    todo: "move".to_owned(),
                    src: "this/is/var2/yes".to_owned(),
                    dst: "this/is/var3/yes".to_owned(),
                },
            ],
        };
        let variables = None;
        let mut ops = vec![Operation::Move, Operation::Hardlink, Operation::Copy_];
        let mut arg_types: Vec<ArgsType> = Vec::new();
        asset_def.jobs.iter().for_each(|d| {
            arg_types.push(ArgsType::CmdLine {
                op: ops.pop().unwrap(),
                to: PathBuf::from(d.dst.clone()),
                from: PathBuf::from(d.src.clone()),
            })
        });
        assert_eq!(arg_types, map_variables(asset_def, variables));
    }

    #[test]
    #[should_panic(expected = "either json file is missing vars, or command line")]
    fn incompatible_vars() {
        let asset_def = AssetRelocationDef {
            variables_in_use: vec![],
            jobs: vec![JobConfigs {
                todo: "copy".to_owned(),
                src: "this/is/var1/yes".to_owned(),
                dst: "this/is/var2/yes".to_owned(),
            }],
        };
        let variables = Some(vec![
            (String::from("var1"), String::from("VAR1")),
            (String::from("var3"), String::from("VAR3")),
        ]);
        let mut ops = vec![Operation::Copy_];
        let mut arg_types: Vec<ArgsType> = Vec::new();
        asset_def.jobs.iter().for_each(|d| {
            arg_types.push(ArgsType::CmdLine {
                op: ops.pop().unwrap(),
                to: PathBuf::from(d.dst.clone()),
                from: PathBuf::from(d.src.clone()),
            })
        });
        assert_eq!(arg_types, map_variables(asset_def, variables));
    }

    #[test]
    #[should_panic(expected = "vars count mismatch")]
    fn vars_count_mismatch() {
        let asset_def = AssetRelocationDef {
            variables_in_use: vec!["var1".to_owned(), "var2".to_owned()],
            jobs: vec![JobConfigs {
                todo: "move".to_owned(),
                src: "this/is/var2/yes".to_owned(),
                dst: "this/is/var3/yes".to_owned(),
            }],
        };
        let variables = Some(vec![(String::from("var1"), String::from("VAR1"))]);
        let mut ops = vec![Operation::Move];
        let mut arg_types: Vec<ArgsType> = Vec::new();
        asset_def.jobs.iter().for_each(|d| {
            arg_types.push(ArgsType::CmdLine {
                op: ops.pop().unwrap(),
                to: PathBuf::from(d.dst.replace("var", "VAR")),
                from: PathBuf::from(d.src.replace("var", "VAR")),
            })
        });
        assert_eq!(arg_types, map_variables(asset_def, variables));
    }

    #[test]
    fn get_json_args_pass() {
        let input_json_args = ArgsType::Json {
            json_file: Path::new("./test_files/asset_relocation_def.json").to_owned(),
            variables: Some(vec![
                (String::from("Configuration"), String::from("debug")),
                (String::from("ProjectName"), String::from("test_proj")),
                (
                    String::from("ProjectDir"),
                    String::from("c:/Users/test/proj_dir"),
                ),
                (
                    String::from("SolutionDir"),
                    String::from("c:/Users/test/sol_dir"),
                ),
                (
                    String::from("OutDir"),
                    String::from("c:/Users/test/out_dir"),
                ),
            ]),
        };
        let out_args = vec![
            ArgsType::CmdLine {
                op: Operation::Hardlink,
                from: PathBuf::from("c:/Users/test/sol_dir/../Bink2/lib/*.dll"),
                to: PathBuf::from("c:/Users/test/out_dir"),
            },
            ArgsType::CmdLine {
                op: Operation::Move,
                from: PathBuf::from("c:/Users/test/proj_dir/assets"),
                to: PathBuf::from("c:/Users/test/out_dir/debug/Games/test_proj"),
            },
        ];
        assert_eq!(out_args, get_json_args(input_json_args));
    }
}
