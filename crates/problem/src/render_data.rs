//! 题面、题解等前端渲染数据（markdown）
//!
//! 渲染数据不需要从设计上保证与评测数据的一致性（因为你也没法保证题面的准确性），需要出题人自己细心维护，
//! 或者借助造题工具来降低错误发生的几率。
//!
//! 渲染数据等源数据仍然与评测数据保存到一起，在 OJ 上在线修改时也需要做出相应的修改。遇到修改也是一样的处理。
//! 数据库里维护的是与搜索有关/已经渲染好的题面等等
//!
//! 不考虑任何修改相关的操作（这些应该由造题工具考虑）

use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::path::PathBuf;

/// 描述一个文件
#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
#[ts(variant_inline)]
pub enum FileDescriptor {
    Stdin,
    Stdout,
    Named(String),
}

impl std::fmt::Display for FileDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileDescriptor::Stdin => write!(f, "stdin"),
            FileDescriptor::Stdout => write!(f, "stdout"),
            FileDescriptor::Named(name) => write!(f, "{}", name),
        }
    }
}

/// for traditional problem
#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
#[ts(variant_inline)]
pub enum IOKind {
    /// read from stdin, write to stdout
    StdIO,
    /// specify named files to read and write
    FileIO {
        input: FileDescriptor,
        output: FileDescriptor,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
#[ts(variant_inline)]
pub enum ProblemKind {
    /// Traditional, also supports NOI style interactive problem
    Traditional(IOKind),
    /// I/O Interactive Problem
    Interactive,
    /// Submit answer only
    SubmitAnswer,
}

/// markdown AST，用于传递给前端
pub type Mdast = md::Node;

pub mod statement {
    use super::*;

    /// 题面数据
    ///
    /// 使用 `[pdf](path)` 的格式插入 pdf
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Statement {
        /// 标题
        pub title: String,
        /// 题面描述
        pub statement: Inner,
        /// 题目元数据
        pub meta: StmtMeta,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum Inner {
        /// custom layout, only for import
        Legacy(String),
        /// standard form, consists of several pre-defined parts
        ///
        /// 与 Legacy 区分的主要原因是样例的格式不是 markdown 语法，依赖自定义的渲染方式
        Standard {
            /// problem background & description
            legend: String,
            /// input format
            input_format: String,
            /// output format
            output_format: String,
            /// notes & constraints & case/subtask specification
            notes: String,
            /// samples, either user input or generated by running a testcase
            samples: Vec<(IOData, IOData)>,
        },
    }

    fn create_heading(content: &str, depth: u8) -> Mdast {
        md::Node::Heading(md::ast::Heading {
            children: vec![md::Node::Text(md::ast::Text {
                value: content.into(),
            })],
            depth,
        })
    }

    impl Inner {
        pub fn render_mdast(&self) -> Mdast {
            match self {
                Inner::Legacy(s) => md::parse_ast(s.as_str()).expect("parse error"),
                Inner::Standard {
                    legend,
                    input_format,
                    output_format,
                    notes,
                    samples,
                } => {
                    let leg = md::parse_ast(legend).expect("parse legend error");
                    let inp = md::parse_ast(input_format).expect("parse input format error");
                    let oup = md::parse_ast(output_format).expect("parse output format error");
                    let notes = md::parse_ast(notes).expect("parse notes error");
                    let mut nodes = Vec::new();
                    if let md::Node::Root(mut r) = leg {
                        nodes.push(create_heading("题目背景", 2));
                        nodes.append(&mut r.children);
                    }
                    if let md::Node::Root(mut r) = inp {
                        nodes.push(create_heading("读入格式", 2));
                        nodes.append(&mut r.children);
                    }
                    if let md::Node::Root(mut r) = oup {
                        nodes.push(create_heading("输出格式", 2));
                        nodes.append(&mut r.children);
                    }
                    if !samples.is_empty() {
                        nodes.push(create_heading("样例", 2));
                        samples.iter().enumerate().for_each(|(id, (i, o))| {
                            nodes.push(create_heading(&format!("样例 #{}", id + 1), 3));
                            nodes.push(md::Node::TwoColumns(md::ast::TwoColumns {
                                left: Box::new(md::Node::Code(md::ast::Code {
                                    value: i.content.clone(),
                                    lang: None,
                                    meta: Some(i.fd.to_string()),
                                })),
                                right: Box::new(md::Node::Code(md::ast::Code {
                                    value: o.content.clone(),
                                    lang: None,
                                    meta: Some(o.fd.to_string()),
                                })),
                            }))
                        })
                    }
                    if let md::Node::Root(mut r) = notes {
                        nodes.push(create_heading("提示", 2));
                        nodes.append(&mut r.children);
                    }
                    md::Node::Root(md::ast::Root { children: nodes })
                }
            }
        }
    }

    /// 样例
    ///
    /// 一个程序与静态数据的交互总是通过文件指针进行的，因此对于样例的结构也可以抽象为此。
    /// 不同类型的题目给出的样例不同。
    ///
    /// 不失一般性，样例只需要考虑可显示的字符（String）
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IOData {
        pub fd: FileDescriptor,
        pub content: String,
    }

    /// 题目显示时的元数据，在渲染 pdf 题面时也会需要
    #[derive(Debug, Clone, Serialize, Deserialize, TsType, Default)]
    pub struct StmtMeta {
        /// 时间限制
        pub time: Option<judger::sandbox::Elapse>,
        /// 空间限制
        pub memory: Option<judger::sandbox::Memory>,
        /// 题目类型
        pub kind: Option<ProblemKind>,
    }
}

pub mod tutorial {
    use super::*;

    /// 题解
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Tutorial {
        pub tutorial: Inner,
        pub meta: TutrMeta,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Inner {
        /// Markdown 文本
        Source(String),
        /// load from an pdf asset
        Pdf(PathBuf),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TutrMeta {
        /// 题目来源、作者
        pub origin: Option<String>,
        /// 有关难度的描述
        pub difficulty: Option<String>,
        /// 算法标签
        pub tags: Vec<String>,
    }
}

pub use statement::Statement;
pub use tutorial::Tutorial;
