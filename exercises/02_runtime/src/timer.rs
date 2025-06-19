// exercises/02_runtime/src/timer.rs
// This implements the TimerFuture example from the async book

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

/// Exercise 2.2: Implement the TimerFuture from the book
/// This timer spawns a background thread and uses Waker to signal completion
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// Shared state between the future and the waiting thread
struct SharedState {
    /// Whether or not the sleep time has elapsed
    completed: bool,
    /// The waker for the task that `TimerFuture` is running on.
    /// The thread can use this after setting `completed = true` to tell
    /// `TimerFuture`'s task to wake up, see that `completed = true`, and
    /// move forward.
    waker: Option<Waker>,
}

impl TimerFuture {
    /// Create a new `TimerFuture` which will complete after the provided timeout.
    pub fn new(duration: Duration) -> Self {
        // TODO: Implement timer creation
        // 1. Create shared state with completed=false, waker=None
        // 2. Spawn a thread that:
        //    - Sleeps for the duration
        //    - Sets completed = true
        //    - Wakes the stored waker if it exists
        // 3. Return TimerFuture with the shared state
        //
        // Hint: Use Arc::new(Mutex::new(SharedState { ... }))
        // Hint: Clone the Arc for the thread with shared_state.clone()
        todo!()
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement the polling logic
        // 1. Lock the shared state
        // 2. Check if completed is true - if so, return Poll::Ready(())
        // 3. If not completed:
        //    - Store the current waker: shared_state.waker = Some(cx.waker().clone())
        //    - Return Poll::Pending
        //
        // IMPORTANT: Always update the waker on each poll!
        // The future might move between different tasks/executors
        todo!()
    }
}

/// Exercise 2.3: Multi-timer that waits for the first to complete
/// This demonstrates racing multiple timers
pub struct FirstTimer {
    timers: Vec<TimerFuture>,
}

impl FirstTimer {
    pub fn new(durations: Vec<Duration>) -> Self {
        // TODO: Create multiple TimerFutures from the durations
        todo!()
    }
}

impl Future for FirstTimer {
    type Output = usize; // Returns the index of the first timer to complete

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Poll all timers and return the index of the first one that's ready
        // Hint: Use Pin::new(&mut timer) to poll each timer
        // This is a simplified version - real select! is more complex
        todo!()
    }
}

/// Exercise 2.4: Cancellable timer
/// A timer that can be cancelled externally
pub struct CancellableTimer {
    timer: Option<TimerFuture>,
    cancelled: Arc<Mutex<bool>>,
}

impl CancellableTimer {
    pub fn new(duration: Duration) -> Self {
        // TODO: Create a timer that can be cancelled
        todo!()
    }

    pub fn cancel(&self) {
        // TODO: Mark this timer as cancelled
        todo!()
    }
}

impl Future for CancellableTimer {
    type Output = Result<(), &'static str>; // Ok(()) if completed, Err if cancelled

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Check if cancelled first, then poll the timer
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_timer_future() {
        let start = Instant::now();
        TimerFuture::new(Duration::from_millis(100)).await;
        let elapsed = start.elapsed();
        
        // Should take approximately 100ms
        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_multiple_timers() {
        let start = Instant::now();
        
        // Create two timers with different durations
        let timer1 = TimerFuture::new(Duration::from_millis(100));
        let timer2 = TimerFuture::new(Duration::from_millis(150));
        
        // Run them concurrently
        let ((), ()) = tokio::join!(timer1, timer2);
        let elapsed = start.elapsed();
        
        // Should take about 150ms (the longer timer)
        assert!(elapsed >= Duration::from_millis(140));
        assert!(elapsed < Duration::from_millis(250));
    }

    #[tokio::test]
    async fn test_first_timer() {
        let start = Instant::now();
        
        let first_timer = FirstTimer::new(vec![
            Duration::from_millis(200),  // index 0
            Duration::from_millis(100),  // index 1 - should complete first
            Duration::from_millis(300),  // index 2
        ]);
        
        let first_index = first_timer.await;
        let elapsed = start.elapsed();
        
        assert_eq!(first_index, 1); // Second timer (100ms) should complete first
        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed < Duration::from_millis(150));
    }

    #[tokio::test]
    async fn test_cancellable_timer() {
        let timer = CancellableTimer::new(Duration::from_millis(200));
        
        // Cancel after 50ms
        let timer_handle = tokio::spawn(async move {
            timer.await
        });
        
        tokio::time::sleep(Duration::from_millis(50)).await;
        // Note: In a real implementation, we'd need a way to cancel the timer
        // For now, just test that uncancelled timers work
        
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            TimerFuture::new(Duration::from_millis(50))
        ).await;
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_timer_waker_update() {
        // This test verifies that the waker is updated on each poll
        use futures::task::{noop_waker, Context};
        
        let timer = TimerFuture::new(Duration::from_millis(100));
        let mut pinned_timer = Box::pin(timer);
        
        // Create two different wakers
        let waker1 = noop_waker();
        let waker2 = noop_waker();
        
        // Poll with first waker
        let mut cx1 = Context::from_waker(&waker1);
        assert_eq!(pinned_timer.as_mut().poll(&mut cx1), Poll::Pending);
        
        // Poll with second waker - should update the stored waker
        let mut cx2 = Context::from_waker(&waker2);
        assert_eq!(pinned_timer.as_mut().poll(&mut cx2), Poll::Pending);
        
        // The test passes if no panic occurs - the internal waker should be updated
    }

    #[tokio::test]
    async fn test_timer_in_select() {
        use tokio::time::timeout;
        
        // Test that our timer works with Tokio's select-like operations
        let result = timeout(
            Duration::from_millis(150),
            TimerFuture::new(Duration::from_millis(100))
        ).await;
        
        assert!(result.is_ok());
        
        // This should timeout
        let result = timeout(
            Duration::from_millis(50),
            TimerFuture::new(Duration::from_millis(100))
        ).await;
        
        assert!(result.is_err());
    }
}