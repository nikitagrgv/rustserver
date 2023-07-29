use std::sync::{mpsc, Arc, LockResult, Mutex};
use std::thread;
use std::thread::JoinHandle;

struct Job {
    func: Box<dyn FnOnce() + Send>,
}

impl Job {
    fn new<F>(f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self { func: Box::new(f) }
    }
}

struct Worker {
    id: usize,
    handle: JoinHandle<()>,
    is_busy: Arc<Mutex<bool>>,
}

impl Worker {
    fn new(id: usize, rx: mpsc::Receiver<Job>) -> Self {
        let is_busy = Arc::new(Mutex::new(false));

        let is_busy_for_thread = is_busy.clone();
        let handle = thread::spawn(move || {
            for j in rx {
                {
                    *is_busy_for_thread.lock().unwrap() = true;
                }
                (j.func)();
                {
                    *is_busy_for_thread.lock().unwrap() = false;
                }
            }
        });
        Self {
            id,
            handle,
            is_busy,
        }
    }

    fn is_busy(&self) -> bool {
        *self.is_busy.lock().unwrap()
    }
}

pub struct ThreadPool {
    workers: Vec<(Worker, mpsc::Sender<Job>)>,
}

impl ThreadPool {
    pub fn new(max_threads: usize) -> Self {
        assert!(max_threads > 0);

        let mut workers = Vec::with_capacity(max_threads);

        for id in 0..max_threads {
            let (tx, rx) = mpsc::channel();
            let worker = Worker::new(id, rx);
            workers.push((worker, tx));
        }

        return Self { workers };
    }

    pub fn run<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Job::new(f);
        let worker = 'l: loop {
            for w in &self.workers {
                println!("BUSINESS: id: {}, busy: {}", w.0.id, w.0.is_busy());


                if !w.0.is_busy() {
                    // println!("NOT BUSY! {}", w.0.id);
                    break 'l &w.1;
                }
            }
        };
        worker.send(job).unwrap();
    }
}
