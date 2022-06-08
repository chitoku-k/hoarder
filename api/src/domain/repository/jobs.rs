use async_trait::async_trait;

#[async_trait]
pub trait JobsRepository: Clone + Send + Sync + 'static {}
