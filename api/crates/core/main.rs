use std::process::ExitCode;

use di::container::Application;

mod di;
mod env;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemallocator")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> ExitCode {
    match Application::start().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("failed to start application\nError: {e:?}");
            ExitCode::FAILURE
        },
    }
}
