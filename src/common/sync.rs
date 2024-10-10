use std::sync::Arc;
use tokio::sync::Mutex;

pub trait Context {
    fn cancel(&mut self);
}
pub type Shared<T> = Arc<Mutex<T>>;
pub struct PtrFac;
impl PtrFac {
    pub fn share<T>(t: T) -> Shared<T> {
        Arc::new(Mutex::new(t))
    }
}
