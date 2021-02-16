use anyhow::Result;
use assert_cmd::Command;
use fs_extra;
use std::path::Path;
use tempfile::TempDir;
use walkdir::WalkDir;

#[test]
fn test_run() -> Result<()> {
    let mut cmd = Command::cargo_bin("esycpy")?;
    let out = String::from_utf8(cmd.output()?.stdout)?;
    assert!(out.contains("rajputrajat@gmail.com"));
    Ok(())
}

#[test]
fn copy_whole_dir_cmdline() -> Result<()> {
    let tmp_dir = TempDir::new().unwrap();
    let src = tmp_dir.path().join("src");
    std::fs::create_dir_all(&src)?;
    let dst = tmp_dir.path().join("dst");
    println!("src: {:?}, dst: {:?}", src, dst);
    let base = Path::new("./test_files/integration_test_env");
    assert!(base.exists());
    fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default())?;
    let mut cmd = Command::cargo_bin("esycpy")?;
    let out = cmd
        .args(&[
            "copy",
            "-s",
            src.to_str().unwrap(),
            "-d",
            dst.to_str().unwrap(),
        ])
        .output()?;
    let dst = dst.join("integration_test_env");
    assert!(out.stdout.len() == 0);
    assert!(out.stderr.len() == 0);
    assert!(dst.join("f1.ext1").exists());
    assert!(dst.join("f5.ext1").exists());
    assert!(dst.join("d2").join("f22.ext1").exists());
    assert!(dst.join("d2").join("d21").join("f211.ext1").exists());
    assert!(dst.join("d2").join("d21").join("f214.ext2").exists());
    assert!(dst
        .join("d2")
        .join("d21")
        .join("d211")
        .join("f2111.ext2")
        .exists());
    assert!(dst.join("d5").join("f51.ext1").exists());
    assert!(dst.join("d5").join("f54.ext2").exists());
    assert!(src.join("integration_test_env").exists());
    Ok(())
}

#[test]
fn move_whole_dir_cmdline() -> Result<()> {
    let tmp_dir = TempDir::new().unwrap();
    let src = tmp_dir.path().join("src");
    std::fs::create_dir_all(&src)?;
    let dst = tmp_dir.path().join("dst");
    println!("src: {:?}, dst: {:?}", src, dst);
    let base = Path::new("./test_files/integration_test_env");
    assert!(base.exists());
    fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default())?;
    let mut cmd = Command::cargo_bin("esycpy")?;
    let out = cmd
        .args(&[
            "move",
            "-s",
            src.to_str().unwrap(),
            "-d",
            dst.to_str().unwrap(),
        ])
        .output()?;
    let dst = dst.join("integration_test_env");
    assert!(out.stdout.len() == 0);
    assert!(out.stderr.len() == 0);
    assert!(dst.join("f1.ext1").exists());
    assert!(dst.join("f5.ext1").exists());
    assert!(dst.join("d2").join("f22.ext1").exists());
    assert!(dst.join("d2").join("d21").join("f211.ext1").exists());
    assert!(dst.join("d2").join("d21").join("f214.ext2").exists());
    assert!(dst
        .join("d2")
        .join("d21")
        .join("d211")
        .join("f2111.ext2")
        .exists());
    assert!(dst.join("d5").join("f51.ext1").exists());
    assert!(dst.join("d5").join("f54.ext2").exists());
    assert!(!src.join("integration_test_env").exists());
    Ok(())
}

#[test]
fn hardlink_whole_dir_cmdline() -> Result<()> {
    let tmp_dir = TempDir::new().unwrap();
    let src = tmp_dir.path().join("src");
    std::fs::create_dir_all(&src)?;
    let dst = tmp_dir.path().join("dst");
    println!("src: {:?}, dst: {:?}", src, dst);
    let base = Path::new("./test_files/integration_test_env");
    assert!(base.exists());
    fs_extra::dir::copy(base, &src, &fs_extra::dir::CopyOptions::default())?;
    let mut cmd = Command::cargo_bin("esycpy")?;
    let out = cmd
        .args(&[
            "hardlink",
            "-s",
            src.to_str().unwrap(),
            "-d",
            dst.to_str().unwrap(),
        ])
        .output()?;
    let dst = dst.join("integration_test_env");
    assert!(out.stdout.len() == 0);
    assert!(out.stderr.len() == 0);
    assert!(dst.join("f1.ext1").exists());
    assert!(dst.join("f5.ext1").exists());
    assert!(dst.join("d2").join("f22.ext1").exists());
    assert!(dst.join("d2").join("d21").join("f211.ext1").exists());
    assert!(dst.join("d2").join("d21").join("f214.ext2").exists());
    assert!(dst
        .join("d2")
        .join("d21")
        .join("d211")
        .join("f2111.ext2")
        .exists());
    assert!(dst.join("d5").join("f51.ext1").exists());
    assert!(dst.join("d5").join("f54.ext2").exists());
    assert!(src.join("integration_test_env").exists());
    Ok(())
}

#[test]
fn json_arg() -> Result<()> {
    let tmp_dir = TempDir::new().unwrap();
    let src = tmp_dir.path().join("src");
    std::fs::create_dir_all(&src)?;
    let dst = tmp_dir.path().join("dst");
    println!("src: {:?}, dst: {:?}", src, dst);
    let base = Path::new("./test_files/integration_test_env");
    assert!(base.exists());
    let copy_option = fs_extra::dir::CopyOptions {
        content_only: true,
        ..Default::default()
    };
    fs_extra::dir::copy(base, &src, &copy_option)?;
    let mut cmd = Command::cargo_bin("esycpy")?;
    let out = cmd
        .args(&[
            "--json",
            "./test_files/integration_test_copier.json",
            "-v",
            &format!("var1={}", src.join("d2").to_str().unwrap()),
            &format!("var2={}", src.to_str().unwrap()),
            &format!("var3={}", src.to_str().unwrap()),
            &format!("var4={}", src.join("d4").to_str().unwrap()),
            &format!("var5={}", dst.join("dst_v5").to_str().unwrap()),
            &format!("var6={}", dst.join("dst_v6").to_str().unwrap()),
        ])
        .output()?;
    println!("{}", String::from_utf8(out.stdout)?);
    println!("{}", String::from_utf8(out.stderr)?);
    let paths: Vec<_> = WalkDir::new(src.parent().unwrap()).into_iter().collect();
    println!("{:#?}", paths);
    assert!(dst
        .join("dst_v5")
        .join("dst_d21")
        .join("f213.ext2")
        .exists());
    assert!(dst
        .join("dst_v5")
        .join("dst_d21")
        .join("f214.ext2")
        .exists());
    assert!(dst
        .join("dst_v5")
        .join("dst_d21")
        .join("f2111.ext2")
        .exists());
    assert!(!dst
        .join("dst_v5")
        .join("dst_d21")
        .join("f211.ext1")
        .exists());

    assert!(dst.join("dst_v5").join("dst_d2").join("f21.ext1").exists());
    assert!(dst.join("dst_v5").join("dst_d2").join("f22.ext1").exists());
    assert!(dst.join("dst_v5").join("dst_d2").join("f23.ext1").exists());
    assert!(!src.join("d2").join("f21.ext1").exists());
    assert!(!src.join("d2").join("f22.ext1").exists());
    assert!(!src.join("d2").join("f23.ext1").exists());

    assert!(dst.join("dst_v6").join("dst_d5").join("f51.ext1").exists());
    assert!(dst.join("dst_v6").join("dst_d5").join("f52.ext1").exists());
    assert!(dst.join("dst_v6").join("dst_d5").join("f53.ext2").exists());
    assert!(dst.join("dst_v6").join("dst_d5").join("f54.ext2").exists());

    assert!(dst
        .join("dst_v6")
        .join("dst_d4")
        .join("d41")
        .join("d411")
        .join("f411.ext1")
        .exists());
    assert!(dst
        .join("dst_v6")
        .join("dst_d4")
        .join("d41")
        .join("d411")
        .join("f412.ext2")
        .exists());

    Ok(())
}
