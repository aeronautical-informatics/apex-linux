#[macro_use]
extern crate log;

use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use apex_hal::prelude::*;
use humantime::format_duration;
use linux_apex_partition::partition::{ApexLinuxPartition, ApexLogger};
use log::LevelFilter;

fn main() {
    ApexLogger::install(LevelFilter::Debug).unwrap();

    Hello.run()
}

struct Hello;

impl Partition<ApexLinuxPartition> for Hello {
    fn cold_start(&self, ctx: &mut StartContext<ApexLinuxPartition>) {
        ctx.create_process(ProcessAttribute {
            period: apex_hal::prelude::SystemTime::Infinite,
            time_capacity: apex_hal::prelude::SystemTime::Infinite,
            entry_point: aperiodic_hello,
            stack_size: 10000,
            base_priority: 1,
            deadline: apex_hal::prelude::Deadline::Soft,
            name: Name::from_str("aperiodic_hello").unwrap(),
        })
        .unwrap()
        .start()
        .unwrap();

        ctx.create_process(ProcessAttribute {
            period: apex_hal::prelude::SystemTime::Normal(Duration::ZERO),
            time_capacity: apex_hal::prelude::SystemTime::Infinite,
            entry_point: periodic_hello,
            stack_size: 10000,
            base_priority: 1,
            deadline: apex_hal::prelude::Deadline::Soft,
            name: Name::from_str("periodic_hello").unwrap(),
        })
        .unwrap()
        .start()
        .unwrap();
    }

    fn warm_start(&self, ctx: &mut StartContext<ApexLinuxPartition>) {
        self.cold_start(ctx)
    }
}

fn aperiodic_hello() {
    for i in 0..i32::MAX {
        if let SystemTime::Normal(time) = Time::<ApexLinuxPartition>::get_time() {
            let round = Duration::from_millis(time.as_millis() as u64);
            info!(
                "{:?}: Aperiodic: Hello {i}",
                format_duration(round).to_string()
            );
        }
        sleep(Duration::from_millis(1))
    }
}

fn periodic_hello() {
    sleep(Duration::from_millis(1));
    //rec(0);
    for i in 0..50 {
        if let SystemTime::Normal(time) = Time::<ApexLinuxPartition>::get_time() {
            let round = Duration::from_millis(time.as_millis() as u64);
            info!(
                "{:?}: Periodic: Hello {i}",
                format_duration(round).to_string()
            );
        }
        sleep(Duration::from_millis(1))
    }
}

fn rec(i: usize) {
    print!("\r{i}");
    rec(i + 1)
}
