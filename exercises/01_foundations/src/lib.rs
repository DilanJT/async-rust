// exercises/01_foundations/src/lib.rs
use std::time::Duration;

/// Exercise 1.1: Make this function async
/// TODO: Convert this to an async function that returns the same string
pub fn hello_async() -> String {
    "Hello, Async World!".to_string()
}

/// Exercise 1.2: Create an async function with delay
/// TODO: Make this function async and add a 1-second delay using tokio::time::sleep
/// Hint: Use tokio::time::sleep(Duration::from_secs(1)).await
pub fn delayed_answer() -> i32 {
    42
}

/// Exercise 1.3: Understanding the difference
/// TODO: Create two versions - one that runs sequentially, one concurrently
pub async fn sequential_work() -> (String, String) {
    // TODO: Call async_task_a().await then async_task_b().await
    // This should take about 2 seconds total
    todo!()
}

pub async fn concurrent_work() -> (String, String) {
    // TODO: Use tokio::join! to run both tasks concurrently
    // This should take about 1 second total (max of both)
    todo!()
}

// Helper functions for exercises
async fn async_task_a() -> String {
    tokio::time::sleep(Duration::from_millis(1000)).await;
    "Task A completed".to_string()
}

async fn async_task_b() -> String {
    tokio::time::sleep(Duration::from_millis(800)).await;
    "Task B completed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_hello_async() {
        let result = hello_async().await;
        assert_eq!(result, "Hello, Async World!");
    }

    #[tokio::test]
    async fn test_delayed_answer() {
        let start = Instant::now();
        let result = delayed_answer().await;
        let duration = start.elapsed();
        
        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(900));
        assert!(duration < Duration::from_millis(1100));
    }

    #[tokio::test]
    async fn test_sequential_vs_concurrent() {
        // Test sequential execution
        let start = Instant::now();
        let (a, b) = sequential_work().await;
        let sequential_duration = start.elapsed();
        
        assert_eq!(a, "Task A completed");
        assert_eq!(b, "Task B completed");
        assert!(sequential_duration >= Duration::from_millis(1700)); // ~1.8s
        
        // Test concurrent execution
        let start = Instant::now();
        let (a, b) = concurrent_work().await;
        let concurrent_duration = start.elapsed();
        
        assert_eq!(a, "Task A completed");
        assert_eq!(b, "Task B completed");
        assert!(concurrent_duration < Duration::from_millis(1200)); // ~1s
        
        // Concurrent should be faster than sequential
        println!("Sequential: {:?}, Concurrent: {:?}", sequential_duration, concurrent_duration);
    }
}