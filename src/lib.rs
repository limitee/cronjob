use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::ops::DerefMut;
use std::str::FromStr;

extern crate chrono;
use chrono::*;

#[derive(Clone, Copy)]
pub enum JobStatus {
    Idle = 0,
    Running,
    Stopped,
}

pub struct CronJob {
    expression: String,
    status: JobStatus,
    second: Vec<i32>,
    minute: Vec<i32>,
    hour: Vec<i32>,
    cur_second: i32,    //当前秒(index)
    cur_minute: i32,    //当前分(index)
    cur_hour: i32,  //当前小时(index)
    cur_day: i32, //当前天
    cur_month: i32,   //当前月
    cur_year: i32,    //当前年

    start_time: DateTime<Local>,    //启动时间
}

impl CronJob {

    pub fn new(exp: String) -> CronJob {
        let mut second = Vec::<i32>::new();
        let mut minute = Vec::<i32>::new();
        let mut hour = Vec::<i32>::new();

        let local: DateTime<Local> = Local::now();  //当前时间
        {
            let exp_pt = &exp;
            let sw = exp_pt.split_whitespace();
            let vec:Vec<&str> = sw.collect();

            if vec[0] == "*" {
                for i in 0..60 {
                    second.push(i);
                }
            } else {
                let sec_array = vec[0].split(',');
                for item in sec_array {
                    second.push(i32::from_str(item).unwrap());
                }
            }

            if vec[1] == "*" {
                for i in 0..60 {
                    minute.push(i);
                }
            } else {
                let min_array = vec[1].split(',');
                for item in min_array {
                    minute.push(i32::from_str(item).unwrap());
                }
            }

            if vec[2] == "*" {
                for i in 0..24 {
                    hour.push(i);
                }
            } else {
                let h_array = vec[2].split(',');
                for item in h_array {
                    hour.push(i32::from_str(item).unwrap());
                }
            }
        }

        CronJob {
            expression: exp,
            status: JobStatus::Idle,
            second: second,
            minute: minute,
            hour: hour,
            cur_second: 0,
            cur_minute: 0,
            cur_hour: 0,
            cur_day: local.day() as i32,
            cur_month: local.month() as i32,
            cur_year: local.year() as i32,
            start_time: local,
        }
    }

    pub fn get_expression(&self) -> String {
        return self.expression.clone();
    }

    /**
     * 跳到正确的位置
     */
    pub fn to_right_position(&mut self) {
        loop {
            let dt = Local.ymd(self.cur_year, self.cur_month as u32, self.cur_day as u32)
                .and_hms(*self.hour.get(self.cur_hour as usize).unwrap() as u32,
                    *self.minute.get(self.cur_minute as usize).unwrap() as u32,
                    *self.second.get(self.cur_second as usize).unwrap() as u32);
            if dt >= self.start_time {
                break;
            } else {
                self.next_tick();
            }
        }
    }

    /**
     * 得到下一个时间点
     */
    pub fn next(&mut self) -> Option<DateTime<Local>> {
        let dt = Local.ymd(self.cur_year, self.cur_month as u32, self.cur_day as u32)
            .and_hms(*self.hour.get(self.cur_hour as usize).unwrap() as u32,
                *self.minute.get(self.cur_minute as usize).unwrap() as u32,
                *self.second.get(self.cur_second as usize).unwrap() as u32);
        if dt > Local::now() {
            return Option::None;
        } else {
            self.next_tick();
            return Option::Some(dt);
        }
    }

    pub fn next_tick(&mut self) {
        if self.cur_second == self.second.len() as i32 - 1 {   //最后s
            if self.cur_minute == self.minute.len() as i32 - 1 {   //最后m
                if self.cur_hour == self.hour.len() as i32 - 1 {   //最后h
                    self.next_day();    //下一天,重置小时,分,秒
                    self.cur_hour = 0;
                    self.cur_minute = 0;
                    self.cur_second = 0;
                } else {    //小时进位,重置分钟,秒
                    self.cur_hour += 1;
                    self.cur_minute = 0;
                    self.cur_second = 0;
                }
            } else {
                self.cur_minute += 1;
                self.cur_second = 0;    //分钟进位,重置秒
            }
        } else {
            self.cur_second += 1;
        }
    }

    /**
     * 下一天的时间
     */
    pub fn next_day(&mut self) {
        if self.cur_day >= self.days_in_month(self.cur_month, self.cur_year) { //一个月的最后一天
            if self.cur_month >= 12 {   //下一年的第一天
                self.cur_year += 1;
                self.cur_month = 1;
                self.cur_day = 1;
            } else {    //下一个月的1号
                self.cur_month += 1;
                self.cur_day = 1;
            }
        }
        else {
            self.cur_day += 1;
        }
    }

    fn is_leap_year(&self, year: i32) -> bool {
        let by_four = year % 4 == 0;
        let by_hundred = year % 100 == 0;
        let by_four_hundred = year % 400 == 0;
        return by_four && ((!by_hundred) || by_four_hundred);
    }

    fn days_in_month(&self, month: i32, year: i32) -> i32 {
        let is_leap_year = self.is_leap_year(year);
        match month {
            9 | 4 | 6 | 11 => 30,
            2 if is_leap_year => 29,
            2 => 28,
            _ => 31
        }
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

    pub fn new(mut job:CronJob) -> JobTracker {
        job.to_right_position();
        JobTracker {
            job: Arc::new(Mutex::new(job)),
        }
    }

    pub fn start<F>(&mut self, mut f:F) where F:FnMut(&mut CronJob, DateTime<Local>) -> bool + 'static {
        let (tx, rx) = mpsc::channel();

        let job = self.job.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            loop {
                let mut job = job.lock().unwrap();

                let status = job.get_status();
                match status {
                    JobStatus::Idle => {
                        match job.next() {
                            Some(x) => {
                                tx.send((x)).unwrap();
                            },
                            None => {
                            },
                        }
                    },
                    JobStatus::Stopped => {
                        println!("stopping from sub thread.");
                        tx.send((Local::now())).unwrap();
                        break;
                    },
                    _ => {
                        break;
                    },
                }

                drop(job);
                thread::sleep(std::time::Duration::new(1, 0));
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