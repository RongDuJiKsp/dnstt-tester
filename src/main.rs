mod client;
mod common;
mod server;
mod application;

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(application::run_application)
}
