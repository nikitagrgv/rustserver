use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
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
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || {
            println!("im thread {id}");
            loop {
                let job = rx.lock().unwrap().recv();
                match job {
                    Ok(job) => (job.func)(),
                    Err(_) => break,
                }
            }
        });
        Self { id, handle }
    }
}

struct WorkerInfo {
    worker: Worker,
}

pub struct ThreadPool {
    workers: Vec<WorkerInfo>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(max_threads: usize) -> Self {
        assert!(max_threads > 0);

        let mut workers = Vec::with_capacity(max_threads);

        let (tx, rx) = mpsc::channel();

        let rx = Arc::new(Mutex::new(rx));

        for id in 0..max_threads {
            let worker = Worker::new(id, rx.clone());
            workers.push(WorkerInfo { worker });
        }

        return Self {
            workers,
            sender: tx,
        };
    }

    pub fn run<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Job::new(f);
        self.sender.send(job).unwrap();
    }
}
