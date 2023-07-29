use std::thread;
use std::thread::JoinHandle;


struct Worker {
    id: usize,
    handle: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize) -> Self {
        let handle = thread::spawn(|| {});
        Self { id, handle }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(max_threads: usize) -> Self {
        assert!(max_threads > 0);

        let mut workers = Vec::with_capacity(max_threads);

        for id in 0..max_threads {
            let worker = Worker::new(id);
            workers.push(worker);
        }

        return Self { workers };
    }

    pub fn run<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        f();
    }
}
