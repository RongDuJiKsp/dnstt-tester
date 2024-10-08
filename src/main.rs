mod application;
mod client;
mod common;
mod server;
fn sync_init() {
    env_logger::init();
}
fn main() {
    sync_init();
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(application::run_application())
}
