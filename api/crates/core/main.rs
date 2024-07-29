use std::process::ExitCode;

use di::container::Application;

mod di;
mod env;

#[cfg(feature = "jemallocator")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> ExitCode {
    match Application::start().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("failed to start application\nError: {e:?}");
            ExitCode::FAILURE
        },
    }
}
