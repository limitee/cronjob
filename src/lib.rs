use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

pub enum JobStatus {
    Idle = 0,
    Running,
}

pub struct CronJob {
    expression: String,
    status: JobStatus,
}

impl CronJob {

    pub fn new(exp: String) -> CronJob {
        CronJob {
            expression: exp,
            status: JobStatus::Idle,
        }
    }

    pub fn has_next(&mut self) -> bool {
        true
    }

}

pub struct JobTracker {
    job: Arc<Mutex<CronJob>>,
}

impl JobTracker {

    pub fn new(job:CronJob) -> JobTracker {
        JobTracker {
            job: Arc::new(Mutex::new(job)),
        }
    }

    pub fn start<F>(&mut self, mut f:F) where F:FnMut(&str) + 'static {
        let (tx, rx) = mpsc::channel();
        let job = self.job.clone();
        thread::spawn(move || {
            loop {
                let mut job = job.lock().unwrap();
                if job.has_next() {
                    tx.send(("abc")).unwrap();
                }
                drop(job);
                thread::sleep_ms(1000);
            }
        });

        loop {
            let back = rx.recv().unwrap();
            let job = self.job.clone();
            let mut job = job.lock();
            f(back);
            drop(job);
        }
    }
}