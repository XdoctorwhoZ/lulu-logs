/// Configuration for the embedded MQTT broker.
#[derive(Debug, Clone)]
pub struct MqttBrokerConfig {
    /// TCP listen address.
    pub addr: String,
    /// TCP port.
    pub port: u16,
    /// Maximum number of simultaneous connections.
    pub max_connections: usize,
    /// Maximum segment size in bytes (100 MB).
    pub max_segment_size: usize,
    /// Enable dynamic filter support.
    pub dynamic_filters: bool,
}

impl Default for MqttBrokerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1".to_string(),
            port: 1883,
            max_connections: 20_480,
            max_segment_size: 100 * 1024 * 1024, // 100 MB
            dynamic_filters: true,
        }
    }
}
