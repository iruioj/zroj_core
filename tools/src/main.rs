use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use inquire::{Select, Text};
use std::{fs, process::Command};

/// 辅助 ZROJ 开发的命令行工具
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    /// 获取当前的项目版本号
    #[arg(short = 'V', long)]
    version: bool,

    #[command(subcommand)]
    command: Commands,
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
enum ConventionalOptions<'a> {
    Feat(&'a str),
    Chore(&'a str),
    Fix(&'a str),
    Docs(&'a str),
    Refactor(&'a str),
    Test(&'a str),
    Amend(&'a str),
    Reject(&'a str),
}

impl<'a> std::fmt::Display for ConventionalOptions<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            ConventionalOptions::Feat(s) => s,
            ConventionalOptions::Chore(s) => s,
            ConventionalOptions::Fix(s) => s,
            ConventionalOptions::Docs(s) => s,
            ConventionalOptions::Refactor(s) => s,
            ConventionalOptions::Test(s) => s,
            ConventionalOptions::Amend(s) => s,
            ConventionalOptions::Reject(s) => s,
        };
        f.write_str(s)
    }
}

fn main() {
    // https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#subcommands
    let args = Cli::parse();

    if args.version {
        let s = fs::read_to_string("Cargo.toml").expect("读取当前目录下的 Cargo.toml 失败");
        let data = s.parse::<toml::Table>().expect("解析 toml 文件出错");
        let version = data
            .get("workspace")
            .expect("没有找到 workspace 字段")
            .get("package")
            .expect("没有找到 workspace.package 字段")
            .get("version")
            .expect("没有找到 workspace.package.version 字段")
            .as_str()
            .expect("版本号不是一个字符串");
        println!("{} {}", "[当前版本]".green(), version);
        return ();
    }

    if let Commands::Commit(_) = args.command {
        use ConventionalOptions::*;
        let options = vec![
            Feat("A) 主要新建了一个子项目"),
            Feat("B) 主要在一个子项目中加入了一些新的函数、类型、traits"),
            Refactor("C) 主要优化（重构）了一下代码实现，删除了一些没啥用的东西"),
            Fix("D) 主要修复了一个 bug"),
            Docs("E) 主要修改了一下文档注释或者 Readme 文件"),
            Test("F) 主要添加、修改了一些测试代码"),
            Chore("G) 感觉好像啥都干了，不是特别好分类"),
            // Amend( "H) 当前做的修改和上一次提交干的事情差不多，并且上一次提交没有同步到远程服务器上",),
            Reject(
                "I) 当前做的修改和上一次提交干的事情差不多，但是上一次提交已经同步到远程服务器上",
            ),
            Reject("J) 在多个子项目中都做了大量的修改"),
        ];

        let commit_type = match Select::new("你对项目做的修改可以归纳为：", options)
            .prompt()
            .expect("选择类型时出现错误")
        {
            Feat(_) => "feat",
            Chore(_) => "chore",
            Fix(_) => "fix",
            Docs(_) => "docs",
            Refactor(_) => "refactor",
            Test(_) => "test",
            Amend(_) => {
                println!("未实现");
                return ();
                // println!("本次修改的内容将与上一次的提交合并。");
            }
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

        Command::new("git")
            .args(["commit", "-m", &full_msg])
            .status()
            .expect("执行 git 命令时出错");

        return ();
    }

    println!("未实现！")
}
