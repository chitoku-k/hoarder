use std::process::exit;

use di::container::Application;

mod di;
mod env;

#[tokio::main]
async fn main() {
    match Application::start().await {
        Ok(()) => {
            exit(0);
        },
        Err(e) => {
            log::error!("failed to start application\nError: {e:?}");
            exit(1);
        },
    }
}
