extern crate cronjob;
use cronjob::*;

extern crate chrono;
use chrono::*;

fn main() {
    let utc: DateTime<UTC> = UTC::now();       // e.g. `2014-11-28T12:45:59.324310806Z`
    let local: DateTime<Local> = Local::now(); // e.g. `2014-11-28T21:45:59.324310806+09:00`

    println!("{}", utc);
    println!("{}", local);

    let mut cj = CronJob::new(String::from("00 * *"));
    let mut jt = JobTracker::new(cj);
    jt.start(move |time| {
        println!("{}", time);
    });

}
