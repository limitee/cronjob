extern crate cronjob;
use cronjob::*;

fn main() {
    let cj = CronJob::new(String::from("02,59 39 17"));
    let mut jt = JobTracker::new(cj);
    jt.start(move |_, time| {
        println!("{}", time);
        true
    });
}
