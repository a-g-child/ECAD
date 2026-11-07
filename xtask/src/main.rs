//! Portable, offline local verification; run `cargo xtask full` from the workspace.

mod boundaries;

use std::{
    error::Error,
    path::Path,
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("check failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.len() > 1 {
        return Err("usage: cargo xtask [quick|full|scale|ui|desktop]".into());
    }
    let profile = arguments.first().map(String::as_str).unwrap_or("full");
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("workspace root is missing")?,
    )?;
    if matches!(profile, "ui" | "desktop") {
        if profile == "ui" {
            cargo(&["test", "--offline", "--locked", "-p", "ecad-bridge"])?;
        }
        npm(&["run", "format:check"])?;
        npm(&["run", "build"])?;
        if profile == "ui" {
            npm(&["test"])?;
        } else {
            cargo(&[
                "fmt",
                "--manifest-path",
                "apps/desktop/src-tauri/Cargo.toml",
                "--",
                "--check",
            ])?;
            cargo(&[
                "clippy",
                "--offline",
                "--locked",
                "--manifest-path",
                "apps/desktop/src-tauri/Cargo.toml",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ])?;
        }
        println!(
            "{profile} checks passed on {} / {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        return Ok(());
    }
    if profile == "scale" {
        return cargo(&[
            "run",
            "--offline",
            "--locked",
            "--release",
            "-p",
            "ecad-adapters",
            "--example",
            "scale",
        ]);
    }
    if !matches!(profile, "quick" | "full") {
        return Err(
            "available profiles: quick, full, scale, ui, desktop; native interaction automation remains pending".into(),
        );
    }
    boundaries::check()?;
    cargo(&["fmt", "--all", "--", "--check"])?;
    cargo(&[
        "clippy",
        "--offline",
        "--locked",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    if profile == "quick" {
        cargo(&["test", "--offline", "--locked", "--workspace", "--tests"])?;
    } else {
        cargo(&[
            "test",
            "--offline",
            "--locked",
            "--workspace",
            "--all-targets",
        ])?;
        cargo(&["test", "--offline", "--locked", "--workspace", "--doc"])?;
        let mut command = cargo_command();
        let existing = std::env::var("RUSTDOCFLAGS").unwrap_or_default();
        command.env("RUSTDOCFLAGS", format!("{existing} -D warnings"));
        command.args(["doc", "--offline", "--locked", "--workspace", "--no-deps"]);
        execute(command)?;
    }
    println!(
        "{profile} checks passed on {} / {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    Ok(())
}

fn cargo_command() -> Command {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
}

fn cargo(arguments: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut command = cargo_command();
    command.args(arguments);
    execute(command)
}

fn execute(mut command: Command) -> Result<(), Box<dyn Error>> {
    println!("Running {command:?}");
    let status = command.status()?;
    if !status.success() {
        return Err(format!("{command:?} exited with {status}").into());
    }
    Ok(())
}

fn npm(arguments: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new(if cfg!(windows) { "npm.cmd" } else { "npm" });
    command.current_dir("apps/desktop").args(arguments);
    execute(command)
}
