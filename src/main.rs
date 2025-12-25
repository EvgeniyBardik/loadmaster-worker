use anyhow::Result;
use lapin::{
    options::*, types::FieldTable, Connection, ConnectionProperties,
};
use log::{error, info};
use std::env;
use tokio;

mod load_test;
mod stats;
mod types;

use load_test::LoadTestExecutor;
use types::LoadTestMessage;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    dotenv::dotenv().ok();

    info!("🚀 LoadMaster Worker starting...");

    // Get RabbitMQ connection details
    let rabbitmq_url = env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string());

    info!("📡 Connecting to RabbitMQ at {}", rabbitmq_url);

    // Connect to RabbitMQ
    let conn = Connection::connect(&rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    info!("✅ Connected to RabbitMQ successfully");

    // Declare queues
    let load_tests_queue = "load_tests";
    let results_queue = "test_results";
    let metrics_queue = "test_metrics";

    channel
        .queue_declare(
            load_tests_queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_declare(
            results_queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_declare(
            metrics_queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    info!("🎧 Waiting for load test messages...");

    // Create consumer
    let mut consumer = channel
        .basic_consume(
            load_tests_queue,
            "loadmaster_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // Process messages
    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let payload = String::from_utf8_lossy(&delivery.data);
                info!("📨 Received message: {}", payload);

                match serde_json::from_str::<LoadTestMessage>(&payload) {
                    Ok(message) => {
                        info!("🧪 Starting load test: {}", message.test_id);

                        let executor = LoadTestExecutor::new(
                            message,
                            channel.clone(),
                            results_queue.to_string(),
                            metrics_queue.to_string(),
                        );

                        // Execute load test in background
                        tokio::spawn(async move {
                            match executor.execute().await {
                                Ok(_) => info!("✅ Load test completed successfully"),
                                Err(e) => error!("❌ Load test failed: {}", e),
                            }
                        });

                        // Acknowledge message
                        delivery
                            .ack(BasicAckOptions::default())
                            .await
                            .expect("Failed to ack");
                    }
                    Err(e) => {
                        error!("❌ Failed to parse message: {}", e);
                        delivery
                            .nack(BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await
                            .expect("Failed to nack");
                    }
                }
            }
            Err(e) => {
                error!("❌ Consumer error: {}", e);
            }
        }
    }

    Ok(())
}

