use std::{ffi::CString, os::unix::ffi::OsStrExt, path::PathBuf, str::FromStr};

use anyhow::Context;
use clap::{error::ErrorKind, CommandFactory, Parser};
use sandbox::{unix::Limitation, ExecSandBox};

/// ZROJ sandbox
#[derive(Parser)]
#[command(
    name = "zroj-sandbox", 
    author,
    disable_version_flag = true,
    about,
    long_about = None,
    styles = sandbox_bin::get_styles(),
)]
struct Cli {
    /// path of the executable
    exec: Option<PathBuf>,

    /// arguments passed to the executable
    args: Vec<String>,

    /// If set, the first argument denotes the name of executable.
    /// Otherwise `exec` is inserted at the begining of the argument list.
    #[arg(long)]
    full_args: bool,

    /// environment variables
    #[arg(short, long, group = "env")]
    envs: Vec<String>,

    /// inherit current environment variables
    #[arg(long, group = "env")]
    pass_env: bool,

    /// redirect stdin from file
    #[arg(long)]
    stdin: Option<PathBuf>,

    /// redirect stdout to file
    #[arg(long)]
    stdout: Option<PathBuf>,

    /// redirect stderr to file
    #[arg(long)]
    stderr: Option<PathBuf>,

    /// resource limitation
    #[arg(long)]
    lim: Option<String>,

    /// save termination status to file, otherwise ignored
    #[arg(short, long, value_name = "FILE")]
    save: Option<PathBuf>,

    /// display version infomation
    #[arg(short = 'V', long)]
    version: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.version {
        sandbox_bin::print_build();
        return Ok(());
    }
    let mut cmd = Cli::command();
    if cli.exec.is_none() {
        cmd.error(
            ErrorKind::MissingRequiredArgument,
            "missing executable path",
        )
        .exit();
    }

    let mut envs = cli.envs;
    if cli.pass_env {
        for (key, value) in std::env::vars() {
            envs.push(format!("{}={}", key, value));
        }
    }

    let mut args = cli.args;
    if !cli.full_args {
        args.insert(0, cli.exec.clone().unwrap().to_str().unwrap().to_string())
    }
    let lim = cli.lim.map(|s| {
        Limitation::from_str(&s)
            .map_err(|e| {
                cmd.error(
                    ErrorKind::InvalidValue,
                    format!("invalid limitation value: {e}"),
                )
                .exit();
            })
            .unwrap()
    });

    let mut s = sandbox::unix::Singleton::new(&cli.exec.unwrap())
        .push_arg(
            args.iter()
                .map(|s| CString::new(s.as_bytes()))
                .collect::<Result<Vec<CString>, _>>()
                .unwrap(),
        )
        .push_env(
            envs.iter()
                .map(|s| CString::new(s.as_bytes()))
                .collect::<Result<Vec<CString>, _>>()
                .unwrap(),
        );
    if let Some(stdin) = cli.stdin {
        s = s.stdin(CString::new(stdin.as_os_str().as_bytes()).unwrap());
    }
    if let Some(stdout) = cli.stdout {
        s = s.stdout(CString::new(stdout.as_os_str().as_bytes()).unwrap());
    }
    if let Some(stderr) = cli.stderr {
        s = s.stderr(CString::new(stderr.as_os_str().as_bytes()).unwrap());
    }
    if let Some(lim) = lim {
        s = s.set_limits(|_| lim)
    }

    let term = s.exec_sandbox().context("execute sandbox")?;

    if let Some(path) = cli.save {
        let file = std::fs::File::create(path).unwrap();
        serde_json::to_writer_pretty(&file, &term).unwrap();
        drop(file)
    }
    Ok(())
}
