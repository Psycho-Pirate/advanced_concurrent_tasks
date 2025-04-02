use anyhow::Result;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{
    sync::broadcast,
    task::JoinHandle,
    time::sleep,
};
use tracing::{error, info};

/// Enum representing possible task errors.
#[derive(Error, Debug)]
pub enum TaskError {
    /// Critical failure in a task, identified by its ID.
    #[error("Critical failure in task {0}")]
    Critical(usize),
    
    /// Non-critical failure in a task, identified by its ID.
    #[error("Non-critical failure in task {0}")]
    NonCritical(usize),
}

/// Struct representing a resource associated with a task.
pub struct Resource {
    id: usize,
}

impl Resource {
    /// Creates a new resource with the given ID.
    pub fn new(id: usize) -> Self {
        info!(task_id = id, "Resource created");
        Self { id }
    }
}

/// Implements cleanup for the resource when it is dropped.
impl Drop for Resource {
    fn drop(&mut self) {
        info!(task_id = self.id, "Resource cleaned up");
    }
}

/// Trait defining a task that can be executed asynchronously.
#[async_trait::async_trait]
pub trait Task: Send + Sync {
    /// Executes the task, checking for shutdown signals.
    async fn execute(&self, shutdown_rx: &mut broadcast::Receiver<()>) -> Result<(), TaskError>;
}

/// Struct representing a worker task that can fail with a given probability.
pub struct WorkerTask {
    id: usize,
    fail_probability: f64,
    duration: Duration,
    critical_failure: bool,
    resource: Resource,
}

impl WorkerTask {
    /// Creates a new worker task.
    /// 
    /// # Arguments
    /// * `id` - The unique identifier for the task.
    /// * `fail_probability` - The probability of failure.
    /// * `duration_ms` - The duration of the task in milliseconds.
    /// * `critical_failure` - Whether failure is critical.
    pub fn new(id: usize, fail_probability: f64, duration_ms: u64, critical_failure: bool) -> Self {
        Self {
            id,
            fail_probability,
            duration: Duration::from_millis(duration_ms),
            critical_failure,
            resource: Resource::new(id),
        }
    }
}

#[async_trait::async_trait]
impl Task for WorkerTask {
    /// Executes the worker task, simulating a failure scenario.
    async fn execute(&self, shutdown_rx: &mut broadcast::Receiver<()>) -> Result<(), TaskError> {
        info!(task_id = self.id, duration = ?self.duration, "Task started");

        let mut rng = StdRng::from_entropy(); 
        let interval = Duration::from_millis(100);
        let elapsed = self.duration.as_millis() / 100;

        for _ in 0..elapsed {
            tokio::select! {
                _ = sleep(interval) => {},
                _ = shutdown_rx.recv() => {
                    info!(task_id = self.id, "Received shutdown signal, stopping early.");
                    return Ok(());
                }
            }
        }
        
        if rng.gen::<f64>() < self.fail_probability {
            if self.critical_failure {
                error!(task_id = self.id, "Critical error occurred");
                return Err(TaskError::Critical(self.id));
            } else {
                error!(task_id = self.id, "Non-critical error occurred");
                return Err(TaskError::NonCritical(self.id));
            }
        }

        info!(task_id = self.id, "Task completed successfully");
        Ok(())
    }
}

/// Struct responsible for managing and executing multiple tasks.
pub struct Orchestrator {
    tasks: Vec<Arc<dyn Task>>,
    shutdown_sender: broadcast::Sender<()>,
}

impl Orchestrator {
    /// Creates a new orchestrator instance.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        Self {
            tasks: Vec::new(),
            shutdown_sender: tx,
        }
    }
    
    /// Adds a task to the orchestrator.
    pub fn add_task(&mut self, task: Arc<dyn Task>) {
        self.tasks.push(task);
    }
    
    /// Runs all added tasks asynchronously, handling failures and shutdown signals.
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting orchestrator with {} tasks", self.tasks.len());
        let mut handles: Vec<JoinHandle<Result<(), TaskError>>> = Vec::new();
        let shutdown_sender = self.shutdown_sender.clone();

        for task in self.tasks.iter() {
            let task_clone = task.clone();
            let mut shutdown_rx = shutdown_sender.subscribe();
            
            let handle = tokio::spawn(async move {
                task_clone.execute(&mut shutdown_rx).await
            });
            
            handles.push(handle);
        }
        
        tokio::select! {
            _ = async {
                for handle in &mut handles {
                    if let Err(TaskError::Critical(id)) = handle.await.unwrap_or_else(|_| Err(TaskError::Critical(0))) {
                        error!("Critical failure in task {}, initiating shutdown.", id);
                        shutdown_sender.send(()).ok();
                        return Err(anyhow::anyhow!("Critical failure in task {}", id));
                    }
                }
                Ok(())
            } => {}
        }

        info!("Orchestrator shutting down...");
        Ok(())
    }
}