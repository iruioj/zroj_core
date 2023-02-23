use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use inquire::{Select, Text};
use semver::Version;
use std::{fs, process::Command, str::FromStr};

/// 辅助 ZROJ 开发的命令行工具
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    /// 获取当前的项目版本号
    #[arg(short = 'V', long)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
    // /// Name of the person to greet
    // #[arg(short, long)]
    // name: String,

    // /// Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 进行交互式的提交
    Commit(CommitArgs),
    Cover,
}

#[derive(Args, Debug)]
struct CommitArgs {}

#[derive(Debug)]
enum VersionChange {
    Minor,
    Patch,
}

#[derive(Debug)]
enum ConventionalOptions<'a> {
    Feat(&'a str, VersionChange),
    Chore(&'a str, VersionChange),
    Fix(&'a str, VersionChange),
    Refactor(&'a str, VersionChange),
    Docs(&'a str),
    Test(&'a str),
    Reject(&'a str),
    // Amend(&'a str),
}

impl<'a> std::fmt::Display for ConventionalOptions<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            ConventionalOptions::Feat(s, _) => s,
            ConventionalOptions::Chore(s, _) => s,
            ConventionalOptions::Fix(s, _) => s,
            ConventionalOptions::Docs(s) => s,
            ConventionalOptions::Refactor(s, _) => s,
            ConventionalOptions::Test(s) => s,
            // ConventionalOptions::Amend(s) => s,
            ConventionalOptions::Reject(s) => s,
        };
        f.write_str(s)
    }
}

fn get_version(data: &toml::Table) -> Option<Version> {
    let ver_str = data
        .get("workspace")?
        .get("package")?
        .get("version")?
        .as_str()?;
    match Version::from_str(ver_str) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

fn main() {
    // https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#subcommands
    let args = Cli::parse();

    let mut cargodata = fs::read_to_string("Cargo.toml")
        .expect("读取当前目录下的 Cargo.toml 失败")
        .parse::<toml::Table>()
        .expect("解析 toml 文件出错");

    if args.version {
        let version = get_version(&cargodata).expect("无法解析版本号");
        println!(
            "{} {}.{}.{}",
            "[当前版本]".green(),
            version.major,
            version.minor,
            version.patch
        );
        return ();
    }

    if let Some(Commands::Commit(_)) = args.command {
        use ConventionalOptions::*;
        let options = vec![
            Feat("A) 主要新建了一个子项目", VersionChange::Minor),
            Feat(
                "B) 主要在一个子项目中加入了一些新的功能、函数、类型、traits",
                VersionChange::Minor,
            ),
            Refactor(
                "C) 主要优化（重构）了一下代码实现、改了改代码格式、删了些没用的东西",
                VersionChange::Patch,
            ),
            Fix(
                "D) 主要修复了一个小 bug（之前运行时没有暴露，自己检查时发现）",
                VersionChange::Patch,
            ),
            Fix(
                "E) 主要修复了一个大 bug（导致运行时出现错误，排查出来的错误）",
                VersionChange::Minor,
            ),
            Docs("F) 主要修改了一下文档注释或者 Readme 文件"),
            Test("G) 主要添加、修改了一些测试代码"),
            Chore(
                "H) 干了些奇奇怪怪的小事情，不是特别好分类",
                VersionChange::Patch,
            ),
            Reject("I) 干了些奇奇怪怪的大事情，不是特别好分类"),
            Reject("K) 在多个子项目中都做了大量的修改"),
            // Amend( "H) 当前做的修改和上一次提交干的事情差不多，并且上一次提交没有同步到远程服务器上",),
            // Reject( "J) 当前做的修改和上一次提交干的事情差不多，但是上一次提交已经同步到远程服务器上",),
        ];

        let (commit_type, ver_change) = match Select::new("你对项目做的修改可以归纳为：", options)
            .prompt()
            .expect("选择类型时出现错误")
        {
            Feat(_, c) => ("feat", Some(c)),
            Chore(_, c) => ("chore", Some(c)),
            Fix(_, c) => ("fix", Some(c)),
            Docs(_) => ("docs", None),
            Refactor(_, c) => ("refactor", Some(c)),
            Test(_) => ("test", None),
            // Amend(_) => {
            //     println!("未实现");
            //     return ();
            //     // println!("本次修改的内容将与上一次的提交合并。");
            // }
            Reject(_) => {
                println!("这种情况无法使用本工具进行自动提交，你可能需要将修改的内容分成几个 commit 来提交。\
                    请你在和其他成员讨论后手动执行 git 命令处理提交。");
                return ();
            }
        };
        let scope = Text::new("你对哪个子项目做出了修改（文件夹名字，如果没有就直接换行）")
            .prompt()
            .expect("输入信息时错误");
        let scope_trimed = scope.trim();

        let message = Text::new("一句话描述你这个提交干了啥（英文）")
            .prompt()
            .expect("输入信息时错误");

        // 开发初期，不考虑 breaking change
        let full_msg = if scope_trimed.len() > 0 {
            format!("{}({}): {}", commit_type, scope_trimed, message)
        } else {
            format!("{}: {}", commit_type, message)
        };

        {
            let ver = cargodata
                .get_mut("workspace")
                .expect("get workspace failed")
                .get_mut("package")
                .expect("get package failed")
                .get_mut("version")
                .expect("get version failed");

            let mut origin_ver = if let toml::Value::String(s) = ver {
                Version::from_str(s).expect("无法解析版本号")
            } else {
                panic!("版本号不是字符串")
            };
            // dbg!(origin_ver.to_string());

            if let Some(vc) = ver_change {
                match vc {
                    VersionChange::Minor => {
                        origin_ver.minor += 1;
                        println!("{}: {}", "[版本更新]".green(), origin_ver.to_string());
                    }
                    VersionChange::Patch => {
                        origin_ver.patch += 1;
                        println!("{}: {}", "[版本更新]".green(), origin_ver.to_string());
                    }
                }
            }
            *ver = toml::Value::String(origin_ver.to_string());
        }

        fs::write("Cargo.toml", cargodata.to_string()).expect("更新 Cargo.toml 时出错");

        Command::new("git")
            .args(["add", "Cargo.toml"])
            .status()
            .expect("执行 git add Cargo.toml 命令时出错");

        Command::new("git")
            .args(["commit", "-m", &full_msg])
            .status()
            .expect("执行 git commit 命令时出错");

        return ();
    }

    println!("未实现！")
}
