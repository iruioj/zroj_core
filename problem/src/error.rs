/// 在创建、修改题目，操作题目数据时出错
#[derive(Debug)]
pub enum DataError {
    IO(std::io::Error),
    Zip(zip::result::ZipError),
    SerdeJson(serde_json::Error),
    NoVersion,
    InvalidVersion,
    InvalidData(String),
    Store(store::Error),
}

impl From<std::io::Error> for DataError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}
impl From<zip::result::ZipError> for DataError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value)
    }
}
impl From<serde_json::Error> for DataError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}
impl From<store::Error> for DataError {
    fn from(value: store::Error) -> Self {
        Self::Store(value)
    }
}

/// 在评测题目时出现的错误，注意这里是指评测错误，不包括选手程序的错误
#[derive(Debug)]
pub enum RuntimeError {
    /// 操作题目数据时出错
    DataError(DataError)
}

impl From<DataError> for RuntimeError {
    fn from(value: DataError) -> Self {
        Self::DataError(value)
    }
}