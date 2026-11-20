use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{JoinHandle, spawn};

type Task = Box<dyn FnOnce() + Send>;

struct State {
    tasks: VecDeque<Task>,
    done: bool,
}

struct Shared {
    state: Mutex<State>,
    cond: Condvar,
}

/// The thread pool.
pub struct Threadpool {
    threads: Vec<JoinHandle<()>>,
    shared: Arc<Shared>,
}

impl Threadpool {
    /// Create new thread pool with `workers_count` workers.
    pub fn new(workers_count: usize) -> Self {
        assert!(workers_count > 0);
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                tasks: VecDeque::new(),
                done: false,
            }),
            cond: Condvar::new(),
        });
        let mut threads = Vec::with_capacity(workers_count);
        for _ in 0..workers_count {
            threads.push(spawn({
                let shared = Arc::clone(&shared);
                move || Self::worker_loop(shared)
            }));
        }
        Self { threads, shared }
    }

    /// Submit a new task.
    pub fn submit(&self, task: Task) {
        {
            let mut state = self.shared.state.lock().expect("Mutex got poisoned");
            state.tasks.push_back(task);
        }
        self.shared.cond.notify_one();
    }

    fn worker_loop(shared: Arc<Shared>) {
        loop {
            let mut state = shared.state.lock().expect("Mutex got poisoned");
            while state.tasks.is_empty() && !state.done {
                state = shared.cond.wait(state).expect("Mutex got poisoned");
            }
            if let Some(task) = state.tasks.pop_front() {
                drop(state);
                task();
            } else {
                // the thread pool is done
                break;
            }
        }
    }
}

impl Drop for Threadpool {
    /// Gracefully end the thread pool.
    ///
    /// It waits until all submitted tasks are executed,
    /// and until all threads are joined.
    fn drop(&mut self) {
        {
            let mut state = self.shared.state.lock().expect("Mutex got poisoned");
            state.done = true;
        }
        self.shared.cond.notify_all();
        for thread in self.threads.drain(..) {
            if let Err(_err) = thread.join() {
                eprintln!("Thread panicked");
            }
        }
    }
}
