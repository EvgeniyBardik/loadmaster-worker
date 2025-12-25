use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoadTestMessage {
    #[serde(rename = "testId")]
    pub test_id: String,
    #[serde(rename = "targetUrl")]
    pub target_url: String,
    pub method: String,
    #[serde(rename = "concurrentUsers")]
    pub concurrent_users: u32,
    #[serde(rename = "totalRequests")]
    pub total_requests: u32,
    #[serde(rename = "durationSeconds")]
    pub duration_seconds: u32,
    #[serde(rename = "requestsPerSecond")]
    pub requests_per_second: u32,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    #[serde(rename = "testId")]
    pub test_id: String,
    #[serde(rename = "totalRequests")]
    pub total_requests: u32,
    #[serde(rename = "successfulRequests")]
    pub successful_requests: u32,
    #[serde(rename = "failedRequests")]
    pub failed_requests: u32,
    #[serde(rename = "averageResponseTime")]
    pub average_response_time: f64,
    #[serde(rename = "minResponseTime")]
    pub min_response_time: f64,
    #[serde(rename = "maxResponseTime")]
    pub max_response_time: f64,
    #[serde(rename = "p50ResponseTime")]
    pub p50_response_time: f64,
    #[serde(rename = "p95ResponseTime")]
    pub p95_response_time: f64,
    #[serde(rename = "p99ResponseTime")]
    pub p99_response_time: f64,
    #[serde(rename = "requestsPerSecond")]
    pub requests_per_second: f64,
    #[serde(rename = "errorRate")]
    pub error_rate: f64,
    #[serde(rename = "statusCodeDistribution")]
    pub status_code_distribution: HashMap<u16, u32>,
    #[serde(rename = "errorDistribution")]
    pub error_distribution: HashMap<String, u32>,
    #[serde(rename = "timeSeriesData")]
    pub time_series_data: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Serialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub rps: f64,
    #[serde(rename = "avgResponseTime")]
    pub avg_response_time: f64,
    #[serde(rename = "errorRate")]
    pub error_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct Metric {
    #[serde(rename = "testId")]
    pub test_id: String,
    pub timestamp: String,
    #[serde(rename = "requestCount")]
    pub request_count: u32,
    #[serde(rename = "successCount")]
    pub success_count: u32,
    #[serde(rename = "errorCount")]
    pub error_count: u32,
    #[serde(rename = "avgResponseTime")]
    pub avg_response_time: f64,
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "activeUsers")]
    pub active_users: u32,
}

