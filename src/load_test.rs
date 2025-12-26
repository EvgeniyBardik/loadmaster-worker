use crate::stats::Statistics;
use crate::types::{LoadTestMessage, Metric, TestResult, TimeSeriesPoint};
use anyhow::Result;
use chrono::Utc;
use lapin::{options::*, Channel};
use log::info;
use reqwest::{Client, Method};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

pub struct LoadTestExecutor {
    message: LoadTestMessage,
    channel: Channel,
    results_queue: String,
    metrics_queue: String,
}

impl LoadTestExecutor {
    pub fn new(
        message: LoadTestMessage,
        channel: Channel,
        results_queue: String,
        metrics_queue: String,
    ) -> Self {
        Self {
            message,
            channel,
            results_queue,
            metrics_queue,
        }
    }

    pub async fn execute(self) -> Result<()> {
        let start_time = Instant::now();
        let stats = Arc::new(tokio::sync::Mutex::new(Statistics::new()));
        
        // Create HTTP client
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        // Semaphore to limit concurrent requests
        let semaphore = Arc::new(Semaphore::new(self.message.concurrent_users as usize));

        // Calculate delay between requests to achieve target RPS
        let delay_between_requests = if self.message.requests_per_second > 0 {
            Duration::from_millis(1000 / self.message.requests_per_second as u64)
        } else {
            Duration::from_millis(10)
        };

        info!(
            "🎯 Target: {} requests @ {} RPS with {} concurrent users",
            self.message.total_requests,
            self.message.requests_per_second,
            self.message.concurrent_users
        );

        let mut handles = vec![];
        let test_duration = Duration::from_secs(self.message.duration_seconds as u64);
        let mut time_series_data = vec![];

        // Execute load test
        for i in 0..self.message.total_requests {
            // Check if duration exceeded
            if start_time.elapsed() >= test_duration {
                info!("⏱️ Duration limit reached, stopping test");
                break;
            }

            let permit = semaphore.clone().acquire_owned().await?;
            let client = client.clone();
            let stats_clone = stats.clone();
            let message = self.message.clone();

            let handle = tokio::spawn(async move {
                let request_start = Instant::now();

                // Parse HTTP method
                let method = Method::from_bytes(message.method.as_bytes())
                    .unwrap_or(Method::GET);

                // Build request
                let mut request_builder = client
                    .request(method, &message.target_url);

                // Add headers if provided
                if let Some(headers) = &message.headers {
                    for (key, value) in headers {
                        request_builder = request_builder.header(key, value);
                    }
                }

                // Add body if provided
                if let Some(body) = &message.body {
                    request_builder = request_builder.json(body);
                }

                // Execute request
                match request_builder.send().await {
                    Ok(response) => {
                        let status = response.status();
                        let response_time = request_start.elapsed().as_millis() as u64;

                        let mut stats = stats_clone.lock().await;
                        stats.record_success(response_time, status.as_u16());
                    }
                    Err(e) => {
                        let mut stats = stats_clone.lock().await;
                        stats.record_failure(e.to_string());
                    }
                }

                drop(permit);
            });

            handles.push(handle);

            // Delay between requests to control RPS
            if (i + 1) % self.message.requests_per_second == 0 {
                sleep(delay_between_requests).await;
            }

            // Send metrics every second
            if (i + 1) % self.message.requests_per_second == 0 {
                let stats_snapshot = stats.lock().await;
                let rps = stats_snapshot.total_requests as f64 / start_time.elapsed().as_secs_f64();
                
                time_series_data.push(TimeSeriesPoint {
                    timestamp: Utc::now().timestamp(),
                    rps,
                    avg_response_time: stats_snapshot.get_average(),
                    error_rate: stats_snapshot.error_rate(),
                });

                // Send metric to queue
                let metric = Metric {
                    test_id: self.message.test_id.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    request_count: stats_snapshot.total_requests,
                    success_count: stats_snapshot.successful_requests,
                    error_count: stats_snapshot.failed_requests,
                    avg_response_time: stats_snapshot.get_average(),
                    status_code: None,
                    error_message: None,
                    active_users: self.message.concurrent_users,
                };

                if let Ok(payload) = serde_json::to_vec(&metric) {
                    let _ = self.channel
                        .basic_publish(
                            "",
                            &self.metrics_queue,
                            BasicPublishOptions::default(),
                            &payload,
                            lapin::BasicProperties::default(),
                        )
                        .await;
                }
            }
        }

        // Wait for all requests to complete
        for handle in handles {
            let _ = handle.await;
        }

        let total_duration = start_time.elapsed();
        let final_stats = stats.lock().await;

        info!(
            "✅ Test completed: {} requests in {:.2}s",
            final_stats.total_requests,
            total_duration.as_secs_f64()
        );

        // Create final test result
        let result = TestResult {
            test_id: self.message.test_id.clone(),
            total_requests: final_stats.total_requests,
            successful_requests: final_stats.successful_requests,
            failed_requests: final_stats.failed_requests,
            average_response_time: final_stats.get_average(),
            min_response_time: final_stats.get_min(),
            max_response_time: final_stats.get_max(),
            p50_response_time: final_stats.get_percentile(50.0),
            p95_response_time: final_stats.get_percentile(95.0),
            p99_response_time: final_stats.get_percentile(99.0),
            requests_per_second: final_stats.total_requests as f64 / total_duration.as_secs_f64(),
            error_rate: final_stats.error_rate(),
            status_code_distribution: final_stats.get_status_codes(),
            error_distribution: final_stats.get_errors(),
            time_series_data,
        };

        // Send result to queue
        let payload = serde_json::to_vec(&result)?;
        self.channel
            .basic_publish(
                "",
                &self.results_queue,
                BasicPublishOptions::default(),
                &payload,
                lapin::BasicProperties::default(),
            )
            .await?;

        info!("📤 Test result sent to queue");

        Ok(())
    }
}

