use assert_cmd::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn success_2_runs() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ronde")?;

    // Generate a temporary directory to store everything
    let temp_dir = tempfile::tempdir()?;
    let dir_path = PathBuf::from(temp_dir.path());
    let binding = dir_path.into_os_string();
    let dir = binding.to_str().unwrap();
    let mut filepath = PathBuf::from(temp_dir.path());
    filepath.push("config.yaml");
    let mut cfg_file = std::fs::File::create(&filepath)?;
    cfg_file.write_all(
        format!(
            r#"---
name: "Ronde"
output_dir: "{}"
history_file: "{}/history.yaml"
commands:
  - name: ping localhost
    run: ping -c 4 localhost
    timeout: 5
  - name: ping not joinable
    run: ping -c 4 notjoinable.notjoinable
    timeout: 5
  - name: ping hits timeout
    run: ping -c 10 localhost
    timeout: 5
"#,
            dir, dir
        )
        .as_bytes(),
    )?;
    cfg_file.sync_all()?;
    drop(cfg_file);

    cmd.arg(filepath.to_str().unwrap());
    cmd.assert().success();

    // Run twice
    cmd.assert().success();
    Ok(())
}
