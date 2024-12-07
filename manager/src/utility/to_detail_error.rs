use std::error::Error;

pub trait ToDetailError {
    fn to_detail_error(self) -> String;
}

impl<T: Error> ToDetailError for T {
    fn to_detail_error(self) -> String {
        format!("{:#?}", self)
    }
}