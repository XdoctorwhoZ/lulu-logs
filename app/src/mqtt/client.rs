use anyhow::Result;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};

use super::rand::generate_random_string;

/// MQTT client wrapper for the lulu-bench viewer.
pub struct PzaMqttClient {
    pub client: AsyncClient,
    pub event_loop: EventLoop,
}

impl PzaMqttClient {
    /// Creates a new MQTT client connected to the given broker.
    ///
    /// The client ID includes a random suffix to avoid ID collisions.
    pub fn new(host: &str, port: u16) -> Self {
        let client_id = format!("lulu-viewer-{}", generate_random_string(5));
        let mut mqtt_options = MqttOptions::new(client_id, host, port);
        mqtt_options.set_keep_alive(std::time::Duration::from_secs(5));

        let (client, event_loop) = AsyncClient::new(mqtt_options, 100);

        Self { client, event_loop }
    }

    /// Subscribes to `lulu/#` to receive all lulu-logs messages.
    pub async fn subscribe_lulu(&self) -> Result<()> {
        self.client
            .subscribe("lulu/#", QoS::AtMostOnce)
            .await?;
        Ok(())
    }
}
