use {Mutex, Condvar};

/// A barrier enables multiple tasks to synchronize the beginning
/// of some computation.
///
/// ```rust
/// use std::sync::Arc;
/// use sync::Barrier;
///
/// let barrier = Arc::new(Barrier::new(10));
/// for _ in range(0u, 10) {
///     let c = barrier.clone();
///     // The same messages will be printed together.
///     // You will NOT see any interleaving.
///     spawn(proc() {
///         println!("before wait");
///         c.wait();
///         println!("after wait");
///     });
/// }
/// ```
pub struct Barrier {
    lock: Mutex<BarrierState>,
    cvar: Condvar,
    num_threads: uint,
}

// The inner state of a double barrier
struct BarrierState {
    count: uint,
    generation_id: uint,
}

impl Barrier {
    /// Create a new barrier that can block a given number of threads.
    ///
    /// A barrier will block `n`-1 threads which call `wait` and then wake up
    /// all threads at once when the `n`th thread calls `wait`.
    pub fn new(n: uint) -> Barrier {
        Barrier {
            lock: Mutex::new(BarrierState {
                count: 0,
                generation_id: 0,
            }),
            cvar: Condvar::new(),
            num_threads: n,
        }
    }

    /// Block the current thread until all tasks has rendezvoused here.
    ///
    /// Barriers are re-usable after all tasks have rendezvoused once, and can
    /// be used continuously.
    pub fn wait(&self) {
        let mut lock = self.lock.lock();
        let local_gen = lock.generation_id;
        lock.count += 1;
        if lock.count < self.num_threads {
            // We need a while loop to guard against spurious wakeups.
            // http://en.wikipedia.org/wiki/Spurious_wakeup
            while local_gen == lock.generation_id &&
                  lock.count < self.num_threads {
                self.cvar.wait(&lock);
            }
        } else {
            lock.count = 0;
            lock.generation_id += 1;
            self.cvar.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::comm::Empty;
    use Barrier;

    #[test]
    fn test_barrier() {
        let barrier = Arc::new(Barrier::new(10));
        let (tx, rx) = channel();

        for _ in range(0u, 9) {
            let c = barrier.clone();
            let tx = tx.clone();
            spawn(proc() {
                c.wait();
                tx.send(true);
            });
        }

        // At this point, all spawned tasks should be blocked,
        // so we shouldn't get anything from the port
        assert!(match rx.try_recv() {
            Err(Empty) => true,
            _ => false,
        });

        barrier.wait();
        // Now, the barrier is cleared and we should get data.
        for _ in range(0u, 9) {
            rx.recv();
        }
    }
}
