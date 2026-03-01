mod app;
mod components;
mod generated;
mod models;
mod mqtt;

use app::App;
use mqtt::{start_broker_in_thread, MqttBrokerConfig};

fn main() {
    // Initialize tracing subscriber (respects RUST_LOG env variable)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Start the embedded MQTT broker in a background thread
    start_broker_in_thread(MqttBrokerConfig::default());

    // Wait for the broker socket to be ready
    std::thread::sleep(std::time::Duration::from_millis(500));

    tracing::info!("Launching lulu-bench UI…");

    // Launch the Dioxus desktop application
    dioxus::launch(App);
}
