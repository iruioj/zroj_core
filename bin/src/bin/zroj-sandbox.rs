use std::{path::PathBuf, str::FromStr};

use clap::{error::ErrorKind, CommandFactory, Parser};
use sandbox::{unix::Limitation, Builder, ExecSandBox};
use shadow_rs::shadow;

shadow!(build);

pub fn print_build() {
    // println!("version:{}", build::VERSION);
    println!("version:{}", build::CLAP_LONG_VERSION);
    // println!("pkg_version:{}", build::PKG_VERSION);
    // println!("pkg_version_major:{}", build::PKG_VERSION_MAJOR);
    // println!("pkg_version_minor:{}", build::PKG_VERSION_MINOR);
    // println!("pkg_version_patch:{}", build::PKG_VERSION_PATCH);
    // println!("pkg_version_pre:{}", build::PKG_VERSION_PRE);

    // println!("tag:{}", build::TAG);
    // println!("branch:{}", build::BRANCH);
    // println!("commit_id:{}", build::COMMIT_HASH);
    // println!("short_commit:{}", build::SHORT_COMMIT);
    // println!("commit_date:{}", build::COMMIT_DATE);
    // println!("commit_date_2822:{}", build::COMMIT_DATE_2822);
    // println!("commit_date_3339:{}", build::COMMIT_DATE_3339);
    // println!("commit_author:{}", build::COMMIT_AUTHOR);
    // println!("commit_email:{}", build::COMMIT_EMAIL);

    // println!("build_os:{}", build::BUILD_OS);
    // println!("rust_version:{}", build::RUST_VERSION);
    // println!("rust_channel:{}", build::RUST_CHANNEL);
    // println!("cargo_version:{}", build::CARGO_VERSION);
    // println!("cargo_tree:{}", build::CARGO_TREE);

    // println!("project_name:{}", build::PROJECT_NAME);
    // println!("build_time:{}", build::BUILD_TIME);
    // println!("build_time_2822:{}", build::BUILD_TIME_2822);
    // println!("build_time:{}", build::BUILD_TIME_3339);
    // println!("build_rust_channel:{}", build::BUILD_RUST_CHANNEL);
}

/// ZROJ sandbox
#[derive(Parser)]
#[command(name = "zroj-sandbox", author, disable_version_flag = true, about, long_about = None)]
struct Cli {
    /// executable path
    exec: Option<PathBuf>,

    /// arguments
    args: Vec<String>,

    /// specify full arguments. i. e. the first argument denotes the name of executable
    /// if unset, the name will be set to the path of executable
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

fn main() {
    let cli = Cli::parse();

    if cli.version {
        print_build();
        return;
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
                cmd.error(ErrorKind::InvalidValue, format!("invalid limitation value: {e}"))
                    .exit();
            })
            .unwrap()
    });

    let mut s = sandbox::unix::SingletonBuilder::new(cli.exec.unwrap())
        .push_arg(args)
        .push_env(envs);
    if let Some(stdin) = cli.stdin {
        s = s.stdin(stdin);
    }
    if let Some(stdout) = cli.stdout {
        s = s.stdout(stdout);
    }
    if let Some(stderr) = cli.stderr {
        s = s.stderr(stderr);
    }
    if let Some(lim) = lim {
        s = s.set_limits(|_| lim)
    }

    let s = s.build().unwrap();

    let term = s.exec_fork().unwrap();

    if let Some(path) = cli.save {
        let file = std::fs::File::create(path).unwrap();
        serde_json::to_writer_pretty(&file, &term).unwrap();
        drop(file)
    }
}
