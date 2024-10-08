use std::fmt::Display;

pub struct Log;
impl Log {
    pub fn error_if_err<T, E: Display>(r: Result<T, E>) {
        if let Err(e) = r {
            log::error!("{}", e)
        }
    }
}
