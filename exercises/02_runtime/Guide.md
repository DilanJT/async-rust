# Module 2: Understanding the Async Runtime 🏗️

## 📖 Prerequisites Reading

Before starting the exercises, read these sections from the async book:

1. **The Future Trait** - Understand `Poll`, `Pending`, and `Ready`
2. **Task Wakeups with Waker** - How futures get woken up
3. **Applied: Build a Timer** - The TimerFuture example
4. **Applied: Build an Executor** - Basic executor implementation

## 🎯 Learning Objectives

By the end of this module, you will:
- [ ] Understand how the `Future` trait works at a low level
- [ ] Implement custom futures from scratch
- [ ] Build a working async executor
- [ ] Understand polling, waking, and task scheduling
- [ ] Know how async runtimes work internally

## 📂 Module Structure

```
exercises/02_runtime/
├── src/
│   ├── lib.rs                  # Module organization
│   ├── custom_future.rs        # Exercise 2.1: Basic futures
│   ├── timer.rs               # Exercise 2.2: Timer with Waker
│   ├── executor.rs            # Exercise 2.3: Build executor
│   └── bin/
│       ├── runtime_demo.rs    # Demo your implementations
│       └── executor_comparison.rs  # Performance testing
└── Cargo.toml
```

## 🚶‍♂️ Step-by-Step Progression

### Exercise 2.1: Custom Futures (custom_future.rs)

**Goal**: Implement the `Future` trait from scratch

**Start Here**:
```bash
cd exercises/02_runtime
cargo test test_countdown_future
```

**Expected Failures**: 
- `todo!()` macros need implementation
- Compilation errors from missing implementations

**Implementation Order**:
1. **CountdownFuture::new()** - Simple initialization
2. **CountdownFuture::poll()** - Basic state machine logic
3. **TimerFuture** - Time-based polling
4. **ManualFuture** - External completion
5. **MapFuture** - Future combinators

**🧠 Key Concepts**:
- `Pin<&mut Self>` - Why futures need pinning
- `Poll::Pending` vs `Poll::Ready` - When to return each
- `Context` and `Waker` - How to register for wake-ups

**💡 Implementation Tips**:

```rust
// CountdownFuture pattern:
impl Future for CountdownFuture {
    type Output = String;
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.count > 0 {
            self.count -= 1;
            Poll::Pending  // Not done yet
        } else {
            Poll::Ready("Countdown complete!".to_string())  // Done!
        }
    }
}
```

**🧪 Testing Your Understanding**:
- Run individual tests: `cargo test countdown`
- Can you explain why `Pin` is needed?
- What happens if you always return `Pending`?

### Exercise 2.2: Timer Future with Waker (timer.rs)

**Goal**: Implement the TimerFuture from the async book

**Start Here**:
```bash
cargo test test_timer_future
```

**This is More Complex**:
- Involves thread spawning
- Proper waker management
- Shared state between thread and future

**Implementation Pattern**:
```rust
// 1. Shared state structure
struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

// 2. Timer creation spawns background thread
impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));
        
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();  // 🔥 This is the magic!
            }
        });
        
        TimerFuture { shared_state }
    }
}

// 3. Polling checks completion and stores waker
impl Future for TimerFuture {
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // IMPORTANT: Always update the waker!
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
```

**🔑 Critical Points**:
- Always update the waker on each poll (futures can move between tasks)
- Thread safety with Arc<Mutex<>>
- Proper cleanup when dropping futures

**🧪 Testing Your Understanding**:
- What happens if you don't call `waker.wake()`?
- Why do we clone the waker on each poll?
- Try implementing `FirstTimer` that races multiple timers

### Exercise 2.3: Build an Executor (executor.rs)

**Goal**: Create a working async executor

**Start Here**:
```bash
cargo test test_basic_executor
```

**This is the Big One**: You're building the heart of an async runtime!

**Architecture Overview**:
```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Spawner   │───▶│ Task Channel │───▶│  Executor   │
└─────────────┘    └──────────────┘    └─────────────┘
                           │                   │
                           ▼                   ▼
                    ┌─────────────┐    ┌─────────────┐
                    │    Task     │    │ Poll Future │
                    │ (Arc<Task>) │    │   & Wake    │
                    └─────────────┘    └─────────────┘
```

**Implementation Steps**:

1. **Channel Setup**:
```rust
pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}
```

2. **Task Structure**:
```rust
struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}
```

3. **Spawning**:
```rust
impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.try_send(task).expect("too many tasks queued");
    }
}
```

4. **Waking**:
```rust
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Send self back to executor queue when woken
        let cloned = arc_self.clone();
        arc_self.task_sender.try_send(cloned).expect("too many tasks queued");
    }
}
```

5. **Execution Loop**:
```rust
impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                
                if future.as_mut().poll(context).is_pending() {
                    // Put it back for later
                    *future_slot = Some(future);
                }
            }
        }
    }
}
```

**🔑 Critical Understanding**:
- **The Task is the Waker**: When a future needs to be woken up, the task sends itself back to the executor
- **Cooperative Scheduling**: Futures must yield control by returning `Pending`
- **No Preemption**: Unlike OS threads, tasks run until they voluntarily yield

**🧪 Testing Your Understanding**:
- Run the demo: `cargo run --bin runtime_demo`
- Can you implement `run_until_stalled()` for testing?
- What happens if a future never returns `Pending`?

### Exercise 2.4: Advanced Executors

**Extend your executor with**:
- **Priority Scheduling**: High/low priority task queues
- **Shutdown Capability**: Graceful executor shutdown
- **Metrics**: Track task execution times

## 🧪 Verification Checklist

Run these to verify your implementations:

```bash
# Test each component
cargo test custom_future
cargo test timer  
cargo test executor

# Run all tests
cargo test

# See your runtime in action
cargo run --bin runtime_demo

# Performance comparison
cargo run --bin executor_comparison
```

## 🔍 Understanding Check

After completing this module, you should be able to answer:

1. **What does `Poll::Pending` mean?** 
   - Future is not ready, will be woken up later

2. **When should you return `Poll::Ready`?**
   - When the future has completed and has a result

3. **What is a Waker and why do we need it?**
   - Mechanism to notify executor that a future can make progress

4. **How does an executor know which task to run next?**
   - Tasks wake themselves up by sending to the executor's queue

5. **What's the difference between blocking and non-blocking?**
   - Blocking stops the whole thread, non-blocking yields to other tasks

## 🚀 Next Steps

Once you've mastered this module:

1. **Read your own code** - You've built a mini-Tokio! 
2. **Compare with real runtimes** - Look at Tokio's source code
3. **Experiment** - Try different scheduling policies
4. **Move to Module 3** - Advanced async/await patterns

## 💡 Common Pitfalls

**❌ Forgetting to update waker**:
```rust
// BAD: Only set waker once
if self.waker.is_none() {
    self.waker = Some(cx.waker().clone());
}

// GOOD: Always update waker
self.waker = Some(cx.waker().clone());
```

**❌ Not handling `Pending` properly**:
```rust
// BAD: Future completes immediately
fn poll(...) -> Poll<...> {
    Poll::Ready(do_work())  // Blocks the executor!
}

// GOOD: Yield control when not ready
fn poll(...) -> Poll<...> {
    if ready {
        Poll::Ready(result)
    } else {
        // Register waker and yield
        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
```

**❌ Forgetting to handle wake-ups**:
```rust
// If you don't implement ArcWake or don't call waker.wake(),
// your tasks will never run again after returning Pending!
```

Your async runtime implementation is a huge milestone - you now understand how async Rust works at the lowest level! 🎉