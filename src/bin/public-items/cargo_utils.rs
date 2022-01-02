use super::Result;

pub fn generate_rustdoc_json_for_current_project() -> Result<()> {
    let mut command = std::process::Command::new("cargo");
    command.env("RUSTDOCFLAGS", "-Z unstable-options --output-format json");
    command.arg("+nightly");
    command.arg("doc");
    command.arg("--lib");
    command.arg("--no-deps");

    eprintln!(
        "Running {:?} with env {:?}",
        command,
        command.get_envs().collect::<Vec<_>>()
    );
    command.spawn()?.wait()?;

    Ok(())
}
