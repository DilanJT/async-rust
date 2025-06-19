// exercises/02_runtime/src/bin/runtime_demo.rs
//! Demo program to showcase your custom runtime implementation
//! 
//! Run with: cargo run --bin runtime_demo

use async_runtime::*;
use std::time::Duration;

fn main() {
    println!("🎯 Async Runtime Demo");
    println!("====================");
    
    // Test 1: Basic custom futures
    println!("\n1️⃣  Testing custom futures:");
    test_custom_futures();
    
    // Test 2: Our executor vs Tokio
    println!("\n2️⃣  Comparing our executor with Tokio:");
    test_executor_comparison();
    
    // Test 3: Advanced executor features
    println!("\n3️⃣  Testing advanced executor features:");
    test_advanced_features();
    
    println!("\n✨ Demo completed! Your runtime is working!");
}

fn test_custom_futures() {
    println!("   🔄 Running countdown future...");
    
    // Use our basic executor
    let (executor, spawner) = new_executor_and_spawner();
    
    spawner.spawn(async {
        let result = CountdownFuture::new(3).await;
        println!("   ✅ {}", result);
    });
    
    spawner.spawn(async {
        println!("   ⏰ Timer starting...");
        TimerFuture::new(Duration::from_millis(100)).await;
        println!("   ✅ Timer completed!");
    });
    
    drop(spawner);
    executor.run();
}

fn test_executor_comparison() {
    use std::time::Instant;
    
    // Test with our executor
    let start = Instant::now();
    let (executor, spawner) = new_executor_and_spawner();
    
    for i in 0..10 {
        spawner.spawn(async move {
            TimerFuture::new(Duration::from_millis(10)).await;
            println!("   📦 Our executor - Task {} completed", i);
        });
    }
    
    drop(spawner);
    executor.run();
    
    let our_time = start.elapsed();
    
    // Test with Tokio (for comparison)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let start = Instant::now();
    
    rt.block_on(async {
        let mut handles = Vec::new();
        for i in 0..10 {
            handles.push(tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                println!("   🚀 Tokio - Task {} completed", i);
            }));
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
    });
    
    let tokio_time = start.elapsed();
    
    println!("   📊 Our executor: {:?}", our_time);
    println!("   📊 Tokio: {:?}", tokio_time);
}

fn test_advanced_features() {
    // Test priority executor if implemented
    println!("   🎯 Testing priority executor...");
    
    let (executor, spawner) = PriorityExecutor::new();
    
    // Spawn some low priority tasks
    for i in 0..3 {
        spawner.spawn_with_priority(async move {
            println!("   🔵 Low priority task {} executed", i);
        }, Priority::Low);
    }
    
    // Spawn some high priority tasks
    for i in 0..3 {
        spawner.spawn_with_priority(async move {
            println!("   🔴 High priority task {} executed", i);
        }, Priority::High);
    }
    
    drop(spawner);
    executor.run();
    
    println!("   ✅ Priority executor test completed!");
}

// exercises/02_runtime/src/bin/executor_comparison.rs
//! Compare different executor implementations
//! 
//! Run with: cargo run --bin executor_comparison

use async_runtime::*;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    println!("⚡ Executor Performance Comparison");
    println!("=================================");
    
    let task_count = 1000;
    let work_duration = Duration::from_micros(100);
    
    println!("\nRunning {} tasks with {}μs work each", task_count, work_duration.as_micros());
    
    // Test our basic executor
    test_our_executor(task_count, work_duration).await;
    
    // Test our priority executor
    test_priority_executor(task_count, work_duration).await;
    
    // Test Tokio for comparison
    test_tokio_executor(task_count, work_duration).await;
}

async fn test_our_executor(task_count: usize, work_duration: Duration) {
    println!("\n🔧 Testing our basic executor:");
    
    let start = Instant::now();
    let counter = Arc::new(AtomicUsize::new(0));
    
    let (executor, spawner) = new_executor_and_spawner();
    
    for _ in 0..task_count {
        let counter_clone = counter.clone();
        spawner.spawn(async move {
            // Simulate some work
            let work_start = Instant::now();
            while work_start.elapsed() < work_duration {
                // Busy wait
            }
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
    }
    
    drop(spawner);
    
    // Run executor in separate thread to avoid blocking
    let handle = tokio::task::spawn_blocking(move || {
        executor.run();
    });
    
    handle.await.unwrap();
    
    let elapsed = start.elapsed();
    let completed = counter.load(Ordering::SeqCst);
    
    println!("   ✅ Completed: {}/{} tasks", completed, task_count);
    println!("   ⏱️  Time: {:?}", elapsed);
    println!("   📊 Tasks/sec: {:.2}", completed as f64 / elapsed.as_secs_f64());
}

async fn test_priority_executor(task_count: usize, work_duration: Duration) {
    println!("\n🎯 Testing priority executor:");
    
    let start = Instant::now();
    let high_counter = Arc::new(AtomicUsize::new(0));
    let low_counter = Arc::new(AtomicUsize::new(0));
    
    let (executor, spawner) = PriorityExecutor::new();
    
    // Half high priority, half low priority
    for i in 0..task_count {
        if i < task_count / 2 {
            let counter_clone = high_counter.clone();
            spawner.spawn_with_priority(async move {
                let work_start = Instant::now();
                while work_start.elapsed() < work_duration {
                    // Busy wait
                }
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }, Priority::High);
        } else {
            let counter_clone = low_counter.clone();
            spawner.spawn_with_priority(async move {
                let work_start = Instant::now();
                while work_start.elapsed() < work_duration {
                    // Busy wait
                }
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }, Priority::Low);
        }
    }
    
    drop(spawner);
    
    let handle = tokio::task::spawn_blocking(move || {
        executor.run();
    });
    
    handle.await.unwrap();
    
    let elapsed = start.elapsed();
    let high_completed = high_counter.load(Ordering::SeqCst);
    let low_completed = low_counter.load(Ordering::SeqCst);
    let total_completed = high_completed + low_completed;
    
    println!("   ✅ High priority: {}", high_completed);
    println!("   ✅ Low priority: {}", low_completed);
    println!("   ✅ Total: {}/{} tasks", total_completed, task_count);
    println!("   ⏱️  Time: {:?}", elapsed);
    println!("   📊 Tasks/sec: {:.2}", total_completed as f64 / elapsed.as_secs_f64());
}

async fn test_tokio_executor(task_count: usize, work_duration: Duration) {
    println!("\n🚀 Testing Tokio executor:");
    
    let start = Instant::now();
    let counter = Arc::new(AtomicUsize::new(0));
    
    let mut handles = Vec::new();
    
    for _ in 0..task_count {
        let counter_clone = counter.clone();
        handles.push(tokio::spawn(async move {
            let work_start = Instant::now();
            while work_start.elapsed() < work_duration {
                // Busy wait
            }
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let elapsed = start.elapsed();
    let completed = counter.load(Ordering::SeqCst);
    
    println!("   ✅ Completed: {}/{} tasks", completed, task_count);
    println!("   ⏱️  Time: {:?}", elapsed);
    println!("   📊 Tasks/sec: {:.2}", completed as f64 / elapsed.as_secs_f64());
}