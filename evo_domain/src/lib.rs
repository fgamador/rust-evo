pub mod biology;
pub mod environment;
pub mod physics;
pub mod world;

use crate::biology::cloud::CloudParameters;
use std::time;

#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    pub cloud_params: CloudParameters,
}

impl Parameters {
    pub const DEFAULT: Parameters = Parameters {
        cloud_params: CloudParameters::DEFAULT,
    };

    // pub fn validate(&self) {
    //     self.cloud_params.validate();
    // }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserAction {
    DebugPrint,
    Exit,
    FastForwardToggle,
    PlayToggle,
    SelectCellToggle { x: f64, y: f64 },
    SingleTick,
}

pub struct ElapsedTimeProbe {
    name: &'static str,
    reporting_interval: time::Duration,
    next_reporting_time: time::Instant,
    probe_begin_time: time::Instant,
    probes_count: u32,
    probes_duration: time::Duration,
}

impl ElapsedTimeProbe {
    pub fn new(name: &'static str, reporting_interval: time::Duration) -> Self {
        ElapsedTimeProbe {
            name,
            reporting_interval,
            next_reporting_time: time::Instant::now() + reporting_interval,
            probe_begin_time: time::Instant::now(),
            probes_count: 0,
            probes_duration: time::Duration::from_secs(0),
        }
    }

    pub fn begin(&mut self) {
        self.probe_begin_time = time::Instant::now();
    }

    pub fn end(&mut self) {
        let end_time = time::Instant::now();
        self.record_probe(end_time.duration_since(self.probe_begin_time));
        if end_time >= self.next_reporting_time {
            self.report();
            self.reset();
        }
    }

    fn record_probe(&mut self, duration: time::Duration) {
        self.probes_count += 1;
        self.probes_duration += duration;
    }

    fn report(&self) {
        println!(
            "{}: {:?}",
            self.name,
            self.probes_duration / self.probes_count
        );
    }

    fn reset(&mut self) {
        self.probes_count = 0;
        self.probes_duration = time::Duration::from_secs(0);
        self.next_reporting_time = time::Instant::now() + self.reporting_interval;
    }
}
