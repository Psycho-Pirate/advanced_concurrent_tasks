// tests/lib_tests.rs
use advanced_concurrent_tasks::{Orchestrator, WorkerTask, Task, TaskError};
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::test]
async fn test_worker_task_success() {
    let task = WorkerTask::new(1, 0.0, 100, false); // No failure expected
    let (tx, mut rx) = broadcast::channel(1);
    let result = task.execute(&mut rx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_worker_task_non_critical_failure() {
    let task = WorkerTask::new(1, 1.0, 100, false); // Always fails, but non-critical
    let (tx, mut rx) = broadcast::channel(1);
    let result = task.execute(&mut rx).await;
    assert!(matches!(result, Err(TaskError::NonCritical(_))));
}

#[tokio::test]
async fn test_worker_task_critical_failure() {
    let task = WorkerTask::new(1, 1.0, 100, true); // Always fails, critical error
    let (tx, mut rx) = broadcast::channel(1);
    let result = task.execute(&mut rx).await;
    assert!(matches!(result, Err(TaskError::Critical(_))));
}

#[tokio::test]
async fn test_orchestrator_run() {
    let mut orchestrator = Orchestrator::new();
    orchestrator.add_task(Arc::new(WorkerTask::new(1, 0.0, 100, false)));
    orchestrator.add_task(Arc::new(WorkerTask::new(2, 0.0, 200, false)));
    orchestrator.add_task(Arc::new(WorkerTask::new(3, 0.1, 150, false)));
    orchestrator.add_task(Arc::new(WorkerTask::new(4, 0.2, 250, false)));
    orchestrator.add_task(Arc::new(WorkerTask::new(5, 0.3, 300, true)));
    let result = orchestrator.run().await;
    assert!(result.is_ok());
}
