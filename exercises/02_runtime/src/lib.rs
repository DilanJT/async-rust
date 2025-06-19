// exercises/02_runtime/src/lib.rs
//! Module 2: Understanding the Async Runtime
//! 
//! This module teaches you how async Rust works under the hood by having you
//! implement the core components yourself:
//! 
//! 1. Custom Future implementations
//! 2. The TimerFuture from the async book  
//! 3. A basic task executor
//! 4. Advanced executor patterns
//!
//! By the end of this module, you'll understand:
//! - How the Future trait works
//! - How polling and waking work
//! - How executors schedule and run tasks
//! - How async runtimes are structured

pub mod custom_future;
pub mod timer;
pub mod executor;

// Re-export main types for easier testing
pub use custom_future::*;
pub use timer::*;
pub use executor::*;

/// A simple example that demonstrates all the concepts together
pub async fn runtime_demo() {
    println!("🚀 Starting runtime demo...");
    
    // Create our own executor
    let (executor, spawner) = new_executor_and_spawner();
    
    // Spawn some tasks using our custom futures
    spawner.spawn(async {
        println!("⏱️  Starting countdown...");
        let result = CountdownFuture::new(3).await;
        println!("✅ {}", result);
    });
    
    spawner.spawn(async {
        println!("⏰ Starting timer...");
        TimerFuture::new(std::time::Duration::from_millis(100)).await;
        println!("✅ Timer completed!");
    });
    
    spawner.spawn(async {
        println!("🔗 Chaining futures...");
        
        // First countdown
        CountdownFuture::new(2).await;
        println!("  First countdown done");
        
        // Then timer
        TimerFuture::new(std::time::Duration::from_millis(50)).await;
        println!("  Timer done");
        
        println!("✅ Chain completed!");
    });
    
    // Drop the spawner so executor knows to exit when tasks are done
    drop(spawner);
    
    println!("🔄 Running executor...");
    executor.run();
    println!("✨ All tasks completed!");
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    use std::time::{Duration, Instant};

    #[test]
    fn test_complete_runtime_integration() {
        // This test combines all the concepts from this module
        let (executor, spawner) = new_executor_and_spawner();
        
        let task_count = Arc::new(AtomicUsize::new(0));
        let start_time = Instant::now();
        
        // Spawn a countdown task
        let task_count_clone = task_count.clone();
        spawner.spawn(async move {
            CountdownFuture::new(3).await;
            task_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        // Spawn a timer task
        let task_count_clone = task_count.clone();
        spawner.spawn(async move {
            TimerFuture::new(Duration::from_millis(100)).await;
            task_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        // Spawn a manual future task
        let task_count_clone = task_count.clone();
        let mut manual = ManualFuture::new();
        spawner.spawn(async move {
            manual.await;
            task_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        // Complete the manual future after a short delay
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            manual.complete(());
        });
        
        // Run the executor in a separate thread
        let executor_handle = std::thread::spawn(move || {
            executor.run();
        });
        
        // Wait for completion
        executor_handle.join().unwrap();
        
        // Verify all tasks completed
        assert_eq!(task_count.load(Ordering::SeqCst), 3);
        
        // Should take roughly 100ms (dominated by the timer)
        let elapsed = start_time.elapsed();
        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_custom_futures_with_tokio() {
        // Test that our custom futures work with Tokio's runtime too
        let start = Instant::now();
        
        let (countdown_result, _timer_result) = tokio::join!(
            CountdownFuture::new(2),
            TimerFuture::new(Duration::from_millis(100))
        );
        
        assert_eq!(countdown_result, "Countdown complete!");
        
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(90));
    }

    #[test]
    fn test_executor_stress() {
        // Spawn many tasks to test executor performance
        let (executor, spawner) = new_executor_and_spawner();
        
        let counter = Arc::new(AtomicUsize::new(0));
        
        // Spawn 1000 simple tasks
        for i in 0..1000 {
            let counter_clone = counter.clone();
            spawner.spawn(async move {
                // Simulate some work
                let _ = i * 2;
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });
        }
        
        drop(spawner);
        
        let start = Instant::now();
        executor.run();
        let elapsed = start.elapsed();
        
        assert_eq!(counter.load(Ordering::SeqCst), 1000);
        
        // Should complete quickly (all CPU-bound work)
        assert!(elapsed < Duration::from_millis(100));
        println!("Executed 1000 tasks in {:?}", elapsed);
    }
}