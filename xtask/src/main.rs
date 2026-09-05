use std::{
    env,
    error::Error,
    path::Path,
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let action = env::args().nth(1).unwrap_or_else(|| "help".into());
    match action.as_str() {
        "check" => {
            cargo(&["fmt", "--all", "--", "--check"])?;
            cargo(&[
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ])?;
            cargo(&["test", "--workspace", "--all-targets", "--locked"])?;
        }
        "headless" => cargo(&[
            "run",
            "-p",
            "aggregate-simulation-core",
            "--example",
            "headless",
            "--locked",
        ])?,
        "help" => println!("cargo run -p xtask -- <check|headless>"),
        _ => return Err(format!("unknown task: {action}").into()),
    }
    Ok(())
}

fn cargo(arguments: &[&str]) -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing workspace parent")?;
    let status = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args(arguments)
        .current_dir(root)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed: {status}", arguments.join(" ")).into())
    }
}
