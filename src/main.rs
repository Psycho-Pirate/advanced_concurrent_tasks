use advanced_concurrent_tasks::{Orchestrator, WorkerTask};
use std::sync::Arc;
use anyhow::Result;
use tracing::{error, info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("Application starting");
    let mut orchestrator = Orchestrator::new();
    
    orchestrator.add_task(Arc::new(WorkerTask::new(1, 0.1, 4000, false)));
    orchestrator.add_task(Arc::new(WorkerTask::new(2, 1.0, 5500, true)));
    orchestrator.add_task(Arc::new(WorkerTask::new(3, 0.0, 7000, false)));
    orchestrator.add_task(Arc::new(WorkerTask::new(4, 0.2, 4500, true)));
    orchestrator.add_task(Arc::new(WorkerTask::new(5, 0.3, 3000, false)));
    
    match orchestrator.run().await {
        Ok(_) => {
            info!("Application completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Application failed: {}", e);
            std::process::exit(1);
        }
    }
}
