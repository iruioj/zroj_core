use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use sandbox::{unix::SingletonConfig, ExecSandBox};

#[derive(Subcommand)]
enum Commands {
    /// Create a default configuration for a command.
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

        /// name of command to be execute
        cmd: String,
        /// arguments passed to the command
        args: Vec<String>,
    },
    /// Execute with JSON config file.
    ///
    /// The output JSON can be deserialized into `Result<sandbox::Termination, Vec<String>>`.
    Run {
        /// path to the JSON config file
        cfg: Option<PathBuf>,
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
            let singleton: SingletonConfig = if let Some(cfg) = cfg {
                let file = std::fs::File::open(cfg)?;
                serde_json::from_reader(file)?
            } else {
                serde_json::from_reader(std::io::stdin())?
            };
            let singleton = singleton.build();
            let term = singleton
                .exec_sandbox()
                .map_err(|e| e.chain().map(|e| e.to_string()).collect::<Vec<String>>());
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
            let mut singleton = SingletonConfig::new(cmd_path)
                .push_args([cmd.as_str()])
                .push_args(args.iter().map(|s| s.as_str()));
            if set_envs {
                singleton = singleton.with_current_env();
            }
            if let Some(stdin) = stdin {
                singleton = singleton.stdin(stdin);
            }
            if let Some(stdout) = stdout {
                singleton = singleton.stdout(stdout);
            }
            serde_json::to_writer_pretty(std::io::stdout(), &singleton)?;
        }
        None => {
            cmd.print_help()?;
            return Ok(());
        }
    }
    Ok(())
}
