use hdrhistogram::Histogram;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Statistics {
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub response_times: Arc<Mutex<Histogram<u64>>>,
    pub status_codes: Arc<Mutex<HashMap<u16, u32>>>,
    pub errors: Arc<Mutex<HashMap<String, u32>>>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            response_times: Arc::new(Mutex::new(
                Histogram::<u64>::new_with_bounds(1, 60000, 3).unwrap(),
            )),
            status_codes: Arc::new(Mutex::new(HashMap::new())),
            errors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record_success(&mut self, response_time_ms: u64, status_code: u16) {
        self.total_requests += 1;
        self.successful_requests += 1;

        let mut histogram = self.response_times.lock().unwrap();
        histogram.record(response_time_ms).ok();

        let mut codes = self.status_codes.lock().unwrap();
        *codes.entry(status_code).or_insert(0) += 1;
    }

    pub fn record_failure(&mut self, error: String) {
        self.total_requests += 1;
        self.failed_requests += 1;

        let mut errors = self.errors.lock().unwrap();
        *errors.entry(error).or_insert(0) += 1;
    }

    pub fn get_percentile(&self, percentile: f64) -> f64 {
        let histogram = self.response_times.lock().unwrap();
        histogram.value_at_percentile(percentile) as f64
    }

    pub fn get_average(&self) -> f64 {
        let histogram = self.response_times.lock().unwrap();
        histogram.mean()
    }

    pub fn get_min(&self) -> f64 {
        let histogram = self.response_times.lock().unwrap();
        histogram.min() as f64
    }

    pub fn get_max(&self) -> f64 {
        let histogram = self.response_times.lock().unwrap();
        histogram.max() as f64
    }

    pub fn get_status_codes(&self) -> HashMap<u16, u32> {
        let codes = self.status_codes.lock().unwrap();
        codes.clone()
    }

    pub fn get_errors(&self) -> HashMap<String, u32> {
        let errors = self.errors.lock().unwrap();
        errors.clone()
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.failed_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}

