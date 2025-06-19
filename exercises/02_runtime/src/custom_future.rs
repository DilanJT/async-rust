// exercises/02_runtime/src/custom_future.rs

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// Exercise 2.1: Implement a simple countdown future
/// This future should count down from a given number to 0
/// Each poll should decrement the counter and return Pending until it reaches 0
pub struct CountdownFuture {
    count: usize,
}

impl CountdownFuture {
    pub fn new(start: usize) -> Self {
        // TODO: Initialize the countdown future
        todo!()
    }
}

impl Future for CountdownFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement the countdown logic
        // - If count > 0, decrement it and return Poll::Pending
        // - If count == 0, return Poll::Ready with "Countdown complete!"
        // 
        // Hint: You can access mutable fields with self.count
        todo!()
    }
}

/// Exercise 2.2: Implement a timer future that actually respects time
/// This should only return Ready after the specified duration has elapsed
pub struct TimerFuture {
    start_time: Option<Instant>,
    duration: Duration,
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        // TODO: Create a timer future
        // Hint: Don't set start_time yet - do it in poll() for lazy initialization
        todo!()
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement timer logic
        // 1. If start_time is None, set it to Instant::now()
        // 2. Check if enough time has elapsed
        // 3. If not, schedule a wake-up and return Pending
        // 4. If yes, return Ready
        //
        // For now, we'll use a simple approach with cx.waker().wake_by_ref()
        // In a real implementation, we'd integrate with an event loop
        todo!()
    }
}

/// Exercise 2.3: A future that can be manually completed
/// This demonstrates how external events can resolve futures
pub struct ManualFuture<T> {
    value: Option<T>,
    waker: Option<std::task::Waker>,
}

impl<T> ManualFuture<T> {
    pub fn new() -> Self {
        // TODO: Create an empty manual future
        todo!()
    }

    pub fn complete(&mut self, value: T) {
        // TODO: Complete the future with a value
        // 1. Store the value
        // 2. If there's a stored waker, wake it
        todo!()
    }
}

impl<T> Future for ManualFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement manual future polling
        // 1. If value is Some, take it and return Ready
        // 2. If value is None, store the waker and return Pending
        todo!()
    }
}

/// Exercise 2.4: A future combinator - maps the output of another future
pub struct MapFuture<F, Func> {
    future: F,
    func: Option<Func>,
}

impl<F, Func> MapFuture<F, Func> {
    pub fn new(future: F, func: Func) -> Self {
        // TODO: Create a map future combinator
        todo!()
    }
}

impl<F, Func, T, U> Future for MapFuture<F, Func>
where
    F: Future<Output = T>,
    Func: FnOnce(T) -> U,
{
    type Output = U;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement the map combinator
        // 1. Poll the inner future
        // 2. If it's Ready, apply the function and return Ready
        // 3. If it's Pending, return Pending
        //
        // SAFETY NOTE: This is a simplified version. Real implementations need
        // to handle pinning properly with unsafe code or pin-project
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use futures::future::poll_fn;
    use std::task::Poll;

    #[test]
    fn test_countdown_future() {
        let future = CountdownFuture::new(3);
        
        // We'll manually poll this future to see the progression
        let mut future = Box::pin(future);
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        // First few polls should return Pending
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        
        // Final poll should return Ready
        assert_eq!(
            future.as_mut().poll(&mut cx), 
            Poll::Ready("Countdown complete!".to_string())
        );
    }

    #[tokio::test]
    async fn test_timer_future() {
        let start = Instant::now();
        TimerFuture::new(Duration::from_millis(100)).await;
        let elapsed = start.elapsed();
        
        // Should take at least 100ms
        assert!(elapsed >= Duration::from_millis(90)); // Small tolerance for timing
    }

    #[test]
    fn test_manual_future() {
        let mut manual_future = ManualFuture::new();
        let mut pinned = Box::pin(&mut manual_future);
        
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        // Should be pending initially
        assert_eq!(pinned.as_mut().poll(&mut cx), Poll::Pending);
        
        // Complete it
        manual_future.complete(42);
        
        // Now should be ready
        assert_eq!(pinned.as_mut().poll(&mut cx), Poll::Ready(42));
    }

    #[tokio::test]
    async fn test_map_future() {
        let original_future = async { 5 };
        let mapped_future = MapFuture::new(original_future, |x| x * 2);
        
        let result = mapped_future.await;
        assert_eq!(result, 10);
    }

    #[tokio::test]
    async fn test_combining_futures() {
        // Test using multiple custom futures together
        let countdown = CountdownFuture::new(2);
        let timer = TimerFuture::new(Duration::from_millis(50));
        
        let (countdown_result, _) = tokio::join!(countdown, timer);
        assert_eq!(countdown_result, "Countdown complete!");
    }
}