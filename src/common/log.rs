pub struct Log;
impl Log {
    pub fn error_if_err<T, E>(r: Result<T, E>) {
        if let Err(e) = r {
            log::error!("{}", e)
        }
    }
}
