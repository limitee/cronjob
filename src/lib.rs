use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::ops::DerefMut;

#[derive(Clone, Copy)]
pub enum JobStatus {
    Idle = 0,
    Running,
    Stopped,
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

    /**
     * 停止任务
     */
    pub fn stop(&mut self) {
        self.status = JobStatus::Stopped;
    }

    /**
     * 获取任务状态
     */
    pub fn get_status(&self) -> JobStatus {
        return self.status;
    }

    /**
     * 设置任务状态
     */
    pub fn set_status(&mut self, status:JobStatus) {
        self.status = status;
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

    pub fn start<F>(&mut self, mut f:F) where F:FnMut(&mut CronJob, i32) -> bool + 'static {
        let (tx, rx) = mpsc::channel();

        let job = self.job.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            loop {
                let mut job = job.lock().unwrap();

                let status = job.get_status();
                match status {
                    JobStatus::Idle => {
                        if job.has_next() {
                            tx.send((1)).unwrap();
                        }
                    },
                    JobStatus::Stopped => {
                        println!("stopping from sub thread.");
                        tx.send((-1)).unwrap();
                        break;
                    },
                    _ => {
                        break;
                    },
                }

                drop(job);
                thread::sleep_ms(1000);
            }
        });

        loop {
            let back = rx.recv().unwrap();
            let job = self.job.clone();
            let mut job = job.lock().unwrap();

            let status = job.get_status();
            match status {
                JobStatus::Idle => {
                    job.set_status(JobStatus::Running);
                    if f(job.deref_mut(), back) {
                        job.set_status(JobStatus::Idle);
                    }
                    else {
                        job.set_status(JobStatus::Stopped);
                    }
                },
                JobStatus::Stopped => {
                    break;
                },
                _ => {

                },
            }
            drop(job);
        }
    }
}