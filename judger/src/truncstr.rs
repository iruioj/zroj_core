use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;

/// 裁剪过的文本内容，用于提交记录中文本文件的展示
#[derive(Debug, Clone, Serialize, Deserialize, TsType)]
pub struct TruncStr {
    str: String,
    limit: usize,
    truncated: usize,
}

impl TruncStr {
    /// 将文本按字符数 <= limit 裁剪
    pub fn new(str: String, limit: usize) -> Self {
        let mut s = String::new();
        let mut counter = limit;
        let mut truncated = 0;
        for c in str.chars() {
            if counter == 0 {
                truncated += 1;
            } else {
                counter -= 1;
                s.push(c);
            }
        }
        Self {
            str: s,
            limit,
            truncated,
        }
    }
}

impl std::fmt::Display for TruncStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}
impl From<&TruncStr> for String {
    fn from(
        TruncStr {
            str,
            limit: _,
            truncated,
        }: &TruncStr,
    ) -> Self {
        if *truncated == 0 {
            str.to_owned()
        } else {
            format!("{str}...({truncated} characters truncated)")
        }
    }
}
impl From<TruncStr> for String {
    fn from(value: TruncStr) -> Self {
        From::from(&value)
    }
}

/// 默认的裁剪长度
pub const TRUNCATE_LEN: usize = 1024;

impl From<String> for TruncStr {
    fn from(value: String) -> Self {
        TruncStr::new(value, TRUNCATE_LEN)
    }
}
impl From<&str> for TruncStr {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}
#[cfg(test)]
mod tests {
    use super::TruncStr;

    #[test]
    fn test_truncstr() {
        let s = TruncStr::new("你好，世界！".to_string(), 10);
        assert_eq!(s.to_string(), "你好，世界！");
        let s = TruncStr::new("你好，世界！".to_string(), 5);
        assert_eq!(s.to_string(), "你好，世界...(1 characters truncated)");
    }
}
