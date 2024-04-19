use std::fmt::Debug;

pub type BusinessResult<T> = Result<T, BusinessError>;

/// 代表一个业务错误
pub struct BusinessError {
    pub reason: String,
}

impl BusinessError {
    pub fn new(reason: impl AsRef<str>) -> Self {
        Self { reason: reason.as_ref().to_owned() }
    }
}

impl<S: AsRef<str>> From<S> for BusinessError {
    fn from(value: S) -> Self {
        BusinessError {
            reason: value.as_ref().to_owned()
        }
    }
}

impl Debug for BusinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.reason)
    }
}

// impl From<std::io::Error> for BusinessError {
//     fn from(value: std::io::Error) -> Self {
        
//     }
// }