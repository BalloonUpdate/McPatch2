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