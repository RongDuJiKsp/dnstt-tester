use anyhow::anyhow;
use std::env;
const SIDE_TIPS: &str = "\
Err: No Side To Run
Program Can Be Run As
./cl client ...args
./cl server ...args\
";
pub async fn run_application() {
    let side = match env::args().skip(1).next() {
        Some(e) => e,
        None => panic!("{}", SIDE_TIPS),
    };
    init_().await.unwrap();
    match side.as_str() {
        "client" => crate::client::application::run_application().await,
        "server" => crate::server::application::run_application().await,
        _ => panic!("{}", SIDE_TIPS),
    };
    println!("{} Exited Successfully", side);
}
async fn init_() -> anyhow::Result<()> {
    Ok(())
}
