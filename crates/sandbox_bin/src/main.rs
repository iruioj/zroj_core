use std::{ffi::CString, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use sandbox::{unix::Singleton, ExecSandBox};
use zroj_sandbox::config::SingletonConfig;

#[derive(Subcommand)]
enum Commands {
    /// create a default configuration for a command
    Show {
        /// input file (redirected to stdin)
        #[arg(long)]
        stdin: Option<String>,
        /// output file (redirected to stdout)
        #[arg(long)]
        stdout: Option<String>,
        /// inhert the env variables of calling process
        #[arg(long)]
        set_envs: bool,

        /// command to be execute
        cmd: String,
        /// arguments passed to the command
        args: Vec<String>,
    },
    /// execute with JSON config file
    Run {
        /// config file path
        cfg: PathBuf,
    },
}

#[derive(Parser)]
#[command(
    name = "zroj-sandbox", 
    author,
    disable_version_flag = true,
    about,
    long_about,
    styles = zroj_sandbox::get_styles(),
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// display version information
    #[arg(short = 'V', long)]
    version: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.version {
        zroj_sandbox::print_build();
        return Ok(());
    }

    let mut cmd = Cli::command();
    match cli.command {
        Some(Commands::Run { cfg }) => {
            let file = std::fs::File::open(cfg)?;
            let singleton: SingletonConfig = serde_json::from_reader(file)?;
            let singleton = Singleton::from(singleton);
            let term = singleton.exec_sandbox()?;
            serde_json::to_writer_pretty(std::io::stdout(), &term)?;
        }
        Some(Commands::Show {
            cmd,
            args,
            stdin,
            stdout,
            set_envs,
        }) => {
            let r = std::process::Command::new("which").arg(&cmd).output()?;
            let cmd_path = String::from_utf8(r.stdout)?;
            let cmd_path = cmd_path.trim();
            let mut singleton = Singleton::new(cmd_path)
                .push_args([CString::new(cmd).unwrap()])
                .push_args(args.into_iter().map(|s| CString::new(s).unwrap()));
            if set_envs {
                singleton = singleton.with_current_env();
            }
            if let Some(stdin) = stdin {
                singleton = singleton.stdin(CString::new(stdin).unwrap());
            }
            if let Some(stdout) = stdout {
                singleton = singleton.stdout(CString::new(stdout).unwrap());
            }
            serde_json::to_writer_pretty(std::io::stdout(), &SingletonConfig::from(singleton))?;
        }
        None => {
            cmd.print_help()?;
            return Ok(());
        }
    }
    Ok(())
}
