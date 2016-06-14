use syncbox::*;
use std::time::Duration;

#[test]
fn test_ordering() {
    let queue = DelayQueue::new();

    queue.offer(Delay(1i32, Duration::from_millis(30))).unwrap();
    queue.offer(Delay(2i32, Duration::from_millis(10))).unwrap();
    queue.offer(Delay(3i32, Duration::from_millis(20))).unwrap();

    assert_eq!(2, *queue.take());
    assert_eq!(3, *queue.take());
    assert_eq!(1, *queue.take());
}

#[test]
fn test_poll() {
    let queue = DelayQueue::new();

    queue.offer(Delay(1i32, Duration::new(0, 0))).unwrap();
    queue.offer(Delay(2i32, Duration::from_secs(86400))).unwrap();

    assert_eq!(1, *queue.poll().unwrap());
    assert_eq!(None, queue.poll());
}

#[test]
fn test_poll_timeout() {
    let queue = DelayQueue::new();

    queue.offer(Delay(1i32, Duration::new(0, 0))).unwrap();
    queue.offer(Delay(2i32, Duration::from_millis(250))).unwrap();
    queue.offer(Delay(3i32, Duration::from_secs(86400))).unwrap();

    assert_eq!(1, *queue.poll_timeout(Duration::from_millis(250)).unwrap());
    assert_eq!(2, *queue.poll_timeout(Duration::from_millis(500)).unwrap());
    assert_eq!(None, queue.poll_timeout(Duration::from_millis(500)));
}
