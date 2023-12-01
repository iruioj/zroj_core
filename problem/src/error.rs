/// 在评测题目时出现的错误，注意这里是指评测错误，不包括选手程序的错误
#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    /// 操作题目数据时出错
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
    #[error("store: {0}")]
    Store(#[from] store::Error),
}
