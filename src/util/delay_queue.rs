use std::collections::BinaryHeap;
use std::cmp::{self, PartialOrd, Ord, PartialEq, Eq, Ordering};
use std::sync::{Mutex, MutexGuard, Condvar};
use std::time::Duration;

use time;

struct Entry<T> {
    t: T,
    time: u64,
}

impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Entry<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Entry<T>) -> Ordering {
        // BinaryHeap is a max heap, so reverse
        self.time.cmp(&other.time).reverse()
    }
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Entry<T>) -> bool {
        self.time == other.time
    }
}

impl<T> Eq for Entry<T> {}

pub struct DelayQueue<T: Send> {
    queue: Mutex<BinaryHeap<Entry<T>>>,
    cvar: Condvar,
}

impl<T: Send> DelayQueue<T> {
    pub fn new() -> DelayQueue<T> {
        DelayQueue {
            queue: Mutex::new(BinaryHeap::new()),
            cvar: Condvar::new(),
        }
    }

    pub fn push(&self, t: T, delay: Duration) {
        let new = Entry {
            t: t,
            time: (time::precise_time_ns() as i64 + delay.num_nanoseconds().unwrap()) as u64,
        };
        let mut queue = self.queue.lock().unwrap();
        match queue.peek() {
            Some(e) if new.time < e.time => self.cvar.notify_all(),
            Some(_) => {}
            None => self.cvar.notify_all(),
        }
        queue.push(new);
    }

    fn finish_pop<'a>(&self, mut queue: MutexGuard<'a, BinaryHeap<Entry<T>>>) -> T {
        if queue.len() > 1 {
            self.cvar.notify_all();
        }
        queue.pop().unwrap().t
    }

    pub fn poll(&self) -> Option<T> {
        let queue = self.queue.lock().unwrap();
        match queue.peek() {
            Some(e) if e.time > time::precise_time_ns() => return None,
            Some(_) => {}
            None => return None,
        }
        Some(self.finish_pop(queue))
    }

    pub fn poll_timeout(&self, timeout: Duration) -> Option<T> {
        let end = (time::precise_time_ns() as i64 + timeout.num_nanoseconds().unwrap()) as u64;
        let mut queue = self.queue.lock().unwrap();
        loop {
            let now = time::precise_time_ns();
            if now >= end {
                return None;
            }

            let wait_until = match queue.peek() {
                Some(e) if e.time <= now => break,
                Some(e) => cmp::min(end, e.time),
                None => end,
            };

            let timeout = Duration::nanoseconds(wait_until as i64 - now as i64);
            queue = self.cvar.wait_timeout(queue, timeout).unwrap().0;
        }

        Some(self.finish_pop(queue))
    }

    pub fn pop(&self) -> T {
        enum Need {
            Wait,
            WaitTimeout(Duration),
        }

        let mut queue = self.queue.lock().unwrap();
        loop {
            let now = time::precise_time_ns();
            let need = match queue.peek() {
                Some(e) if e.time <= now => break,
                Some(e) => Need::WaitTimeout(Duration::nanoseconds(e.time as i64 - now as i64)),
                None => Need::Wait
            };

            queue = match need {
                Need::Wait => self.cvar.wait(queue).unwrap(),
                Need::WaitTimeout(t) => self.cvar.wait_timeout(queue, t).unwrap().0,
            };
        }

        self.finish_pop(queue)
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::DelayQueue;

    #[test]
    fn test_ordering() {
        let queue = DelayQueue::new();

        queue.push(1i32, -Duration::days(1));
        queue.push(2i32, -Duration::days(3));
        queue.push(3i32, -Duration::days(2));

        assert_eq!(2, queue.pop());
        assert_eq!(3, queue.pop());
        assert_eq!(1, queue.pop());
    }

    #[test]
    fn test_poll() {
        let queue = DelayQueue::new();

        queue.push(1i32, Duration::nanoseconds(0));
        queue.push(2i32, Duration::days(1));

        assert_eq!(Some(1), queue.poll());
        assert_eq!(None, queue.poll());
    }

    #[test]
    fn test_poll_timeout() {
        let queue = DelayQueue::new();

        queue.push(1i32, Duration::nanoseconds(0));
        queue.push(2i32, Duration::milliseconds(250));
        queue.push(3i32, Duration::days(1));

        assert_eq!(Some(1), queue.poll_timeout(Duration::milliseconds(250)));
        assert_eq!(Some(2), queue.poll_timeout(Duration::milliseconds(300)));
        assert_eq!(None, queue.poll_timeout(Duration::milliseconds(500)));
    }
}
