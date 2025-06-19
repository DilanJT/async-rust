// exercises/02_runtime/src/executor.rs
// Build your own async executor - the heart of any async runtime

use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::Context,
    time::Duration,
};

/// Exercise 2.3: Implement a basic task executor
/// This executor receives tasks over a channel and runs them to completion
pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
    /// In-progress future that should be pushed to completion.
    ///
    /// The `Mutex` is not necessary for correctness, since we only have
    /// one thread executing tasks at once. However, Rust isn't smart
    /// enough to know that `future` is only mutated from one thread,
    /// so we need to use the `Mutex` to prove thread-safety. A production
    /// executor would not need this, and could use `UnsafeCell` instead.
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    /// Handle to place the task itself back onto the task queue.
    task_sender: SyncSender<Arc<Task>>,
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    // TODO: Create a channel for task communication
    // 1. Use sync_channel with a reasonable buffer size (like 10_000)
    // 2. Return Executor with the receiver and Spawner with the sender
    //
    // Hint: sync_channel returns (SyncSender, Receiver)
    todo!()
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        // TODO: Spawn a future onto the executor
        // 1. Box the future using .boxed()
        // 2. Create a new Task with the boxed future and task_sender
        // 3. Wrap the Task in Arc
        // 4. Send the Arc<Task> through the channel
        //
        // Hint: Use self.task_sender.try_send() and handle the error appropriately
        todo!()
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // TODO: Implement wake behavior
        // When a task is woken, it should be sent back to the executor's queue
        // 1. Clone the Arc<Self>
        // 2. Send it through the task_sender
        // 3. Handle any errors (task queue might be full)
        todo!()
    }
}

impl Executor {
    pub fn run(&self) {
        // TODO: Implement the main executor loop
        // 1. Receive tasks from the ready_queue
        // 2. For each task:
        //    - Take the future from the task (using Mutex::lock and Option::take)
        //    - If there is a future, create a waker and context
        //    - Poll the future
        //    - If it returns Pending, put it back in the task
        //
        // Hint: Use waker_ref(&task) to create a waker
        // Hint: Use BoxFuture::as_mut().poll(context) to poll
        todo!()
    }

    pub fn run_until_stalled(&self) {
        // TODO: Run until no more tasks are ready
        // This is useful for testing - it runs until all tasks are either
        // complete or waiting for external events
        //
        // Hint: Use try_recv() instead of recv() to avoid blocking
        todo!()
    }
}

/// Exercise 2.4: A more advanced executor with task priorities
pub struct PriorityExecutor {
    high_priority: Receiver<Arc<Task>>,
    low_priority: Receiver<Arc<Task>>,
}

#[derive(Clone)]
pub struct PrioritySpawner {
    high_priority_sender: SyncSender<Arc<Task>>,
    low_priority_sender: SyncSender<Arc<Task>>,
}

pub enum Priority {
    High,
    Low,
}

impl PrioritySpawner {
    pub fn spawn_with_priority(
        &self,
        future: impl Future<Output = ()> + 'static + Send,
        priority: Priority,
    ) {
        // TODO: Spawn with different priority levels
        // High priority tasks should be processed before low priority ones
        todo!()
    }
}

impl PriorityExecutor {
    pub fn new() -> (Self, PrioritySpawner) {
        // TODO: Create an executor with two priority levels
        todo!()
    }

    pub fn run(&self) {
        // TODO: Always process high priority tasks first
        // Only process low priority when no high priority tasks are available
        todo!()
    }
}

/// Exercise 2.5: Executor with shutdown capability
pub struct ShutdownExecutor {
    ready_queue: Receiver<Arc<Task>>,
    shutdown_signal: Receiver<()>,
}

pub struct ShutdownSpawner {
    task_sender: SyncSender<Arc<Task>>,
    shutdown_sender: SyncSender<()>,
}

impl ShutdownExecutor {
    pub fn new() -> (Self, ShutdownSpawner) {
        // TODO: Create an executor that can be gracefully shut down
        todo!()
    }

    pub fn run(&self) {
        // TODO: Run until either no more tasks or shutdown signal received
        // Use select! or manual channel checking
        todo!()
    }
}

impl ShutdownSpawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        // TODO: Same as regular spawner
        todo!()
    }

    pub fn shutdown(&self) {
        // TODO: Send shutdown signal
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::TimerFuture;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::Instant;

    #[test]
    fn test_basic_executor() {
        let (executor, spawner) = new_executor_and_spawner();

        // Spawn a simple task
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();

        spawner.spawn(async move {
            executed_clone.store(true, Ordering::SeqCst);
        });

        // Run the executor until no more tasks
        executor.run_until_stalled();

        assert!(executed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_multiple_tasks() {
        let (executor, spawner) = new_executor_and_spawner();

        let counter = Arc::new(AtomicUsize::new(0));

        // Spawn multiple tasks
        for i in 0..5 {
            let counter_clone = counter.clone();
            spawner.spawn(async move {
                counter_clone.fetch_add(i, Ordering::SeqCst);
            });
        }

        executor.run_until_stalled();

        // Sum of 0+1+2+3+4 = 10
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_executor_with_timer() {
        let (executor, spawner) = new_executor_and_spawner();

        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        spawner.spawn(async move {
            TimerFuture::new(Duration::from_millis(50)).await;
            completed_clone.store(true, Ordering::SeqCst);
        });

        // This should complete the timer task
        std::thread::spawn(move || {
            executor.run();
        });

        // Wait a bit for the timer to complete
        std::thread::sleep(Duration::from_millis(100));
        assert!(completed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_nested_spawning() {
        let (executor, spawner) = new_executor_and_spawner();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let spawner_clone = spawner.clone();

        spawner.spawn(async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);

            // Spawn another task from within a task
            spawner_clone.spawn(async move {
                counter_clone.fetch_add(2, Ordering::SeqCst);
            });
        });

        executor.run_until_stalled();
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_priority_executor() {
        let (executor, spawner) = PriorityExecutor::new();

        let order = Arc::new(Mutex::new(Vec::new()));

        // Spawn low priority task first
        let order_clone = order.clone();
        spawner.spawn_with_priority(async move {
            order_clone.lock().unwrap().push("low");
        }, Priority::Low);

        // Then high priority task
        let order_clone = order.clone();
        spawner.spawn_with_priority(async move {
            order_clone.lock().unwrap().push("high");
        }, Priority::High);

        executor.run_until_stalled();

        // High priority should execute first
        let execution_order = order.lock().unwrap();
        assert_eq!(*execution_order, vec!["high", "low"]);
    }

    #[test]
    fn test_shutdown_executor() {
        let (executor, spawner) = ShutdownExecutor::new();

        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        // Spawn a long-running task
        spawner.spawn(async move {
            for _ in 0..1000 {
                tokio::task::yield_now().await;
            }
            completed_clone.store(true, Ordering::SeqCst);
        });

        // Run executor in background
        let handle = std::thread::spawn(move || {
            executor.run();
        });

        // Shutdown immediately
        spawner.shutdown();
        handle.join().unwrap();

        // Task should not have completed due to shutdown
        assert!(!completed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_executor_drop_behavior() {
        // Test that dropping the spawner allows executor to exit gracefully
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        {
            let (executor, spawner) = new_executor_and_spawner();

            spawner.spawn(async move {
                completed_clone.store(true, Ordering::SeqCst);
            });

            executor.run_until_stalled();
            // Spawner dropped here
        }

        assert!(completed.load(Ordering::SeqCst));
    }
}