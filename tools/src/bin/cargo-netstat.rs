use clap::{Parser, Subcommand};
use colored::Colorize;
use tools::netstat;

/// 辅助 ZROJ 开发的命令行工具
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<ExtendCargoCommands>,
}

// 附加到的 cargo 子命令
#[derive(Subcommand, Debug)]
enum ExtendCargoCommands {
    /// Inspect tcp port status
    Netstat,
}

fn main() {
    // https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#subcommands
    match Cli::parse().command {
        None => {
            println!("{}: 请使用 netstat 子命令", "[INFO]".green());
            return;
        }
        Some(args) => match args {
            ExtendCargoCommands::Netstat => {
                for info in netstat() {
                    println!("{}", info);
                }
            }
        },
    };

    println!("{}: 请使用 -h 选项查看帮助", "[INFO]".green());
}
