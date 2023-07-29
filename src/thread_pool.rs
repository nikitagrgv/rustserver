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
    busy: Arc<AtomicBool>,
}

impl Worker {
    fn new(id: usize, rx: mpsc::Receiver<Job>) -> Self {
        let busy = Arc::new(AtomicBool::new(false));

        let busy_th = busy.clone();
        let handle = thread::spawn(move || {
            for j in rx {
                busy_th.store(true, Ordering::Relaxed);
                (j.func)();
                busy_th.store(false, Ordering::Relaxed);
            }
        });
        Self { id, handle, busy }
    }
}

struct WorkerInfo {
    worker: Worker,
    sender: mpsc::Sender<Job>,
}

pub struct ThreadPool {
    workers: Vec<WorkerInfo>,
}

impl ThreadPool {
    pub fn new(max_threads: usize) -> Self {
        assert!(max_threads > 0);

        let mut workers = Vec::with_capacity(max_threads);

        for id in 0..max_threads {
            let (tx, rx) = mpsc::channel();
            let worker = Worker::new(id, rx);

            let worker_info = WorkerInfo { worker, sender: tx };

            workers.push(worker_info);
        }

        return Self { workers };
    }

    pub fn run<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Job::new(f);

        println!("start find");
        let mut worker = self.find_worker();
        println!("worker id = {}", worker.worker.id);

        worker.sender.send(job).unwrap();

        println!("ended");
    }

    fn find_worker<'a>(&'a mut self) -> &'a mut WorkerInfo {
        let idx = 'l: loop {
            for (i, w) in self.workers.iter().enumerate() {
                let busy = w.worker.busy.load(Ordering::Relaxed);
                println!("PREV: {} -  {}", i, busy);
                if !busy {
                    break 'l i;
                }
            }
        };
        return &mut self.workers[idx];
    }
}
