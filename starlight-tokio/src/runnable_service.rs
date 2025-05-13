use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[async_trait]
pub trait StarlightService {
    fn run(&self, shutdown_tx: Arc<watch::Sender<bool>>, shutdown_rx: watch::Receiver<bool>, ) -> JoinHandle<()>;
}