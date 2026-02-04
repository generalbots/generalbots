use cron::Schedule;
use std::str::FromStr;

fn main() {
    let schedules = vec![
        "59 * * * *",
        "0 * * * *",
        "0 11 * * *",
    ];

    for schedule_str in schedules {
        println!("\nTesting: {}", schedule_str);
        match Schedule::from_str(schedule_str) {
            Ok(_) => println!("  ✓ OK"),
            Err(e) => println!("  ✗ Error: {}", e),
        }
    }
}
