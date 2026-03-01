use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};

use rumqttd::{Broker, Config, ConnectionSettings, ServerSettings};

use super::config::MqttBrokerConfig;

/// Starts the embedded MQTT broker in a dedicated thread.
///
/// The broker runs indefinitely in the background. This function returns
/// immediately after spawning the thread.
pub fn start_broker_in_thread(config: MqttBrokerConfig) {
    std::thread::spawn(move || {
        let listen_addr = SocketAddrV4::new(
            config.addr.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::LOCALHOST),
            config.port,
        );

        let server_settings = ServerSettings {
            name: "v4-1".to_string(),
            listen: std::net::SocketAddr::V4(listen_addr),
            tls: None,
            next_connection_delay_ms: 1,
            connections: ConnectionSettings {
                connection_timeout_ms: 5_000,
                max_payload_size: config.max_segment_size,
                max_inflight_count: 100,
                auth: None,
                dynamic_filters: config.dynamic_filters,
                external_auth: None,
            },
        };

        let mut servers = HashMap::new();
        servers.insert("v4-1".to_string(), server_settings);

        let broker_config = Config {
            id: 0,
            router: rumqttd::RouterConfig {
                max_connections: config.max_connections,
                max_segment_size: config.max_segment_size,
                max_outgoing_packet_count: 200,
                max_segment_count: 10,
                ..Default::default()
            },
            v4: Some(servers),
            v5: None,
            ws: None,
            cluster: None,
            console: Some(Default::default()),
            bridge: None,
            prometheus: None,
            metrics: None,
        };

        let mut broker = Broker::new(broker_config);
        tracing::info!(
            "Starting embedded MQTT broker on {}:{}",
            config.addr,
            config.port
        );
        if let Err(e) = broker.start() {
            tracing::error!("MQTT broker exited with error: {}", e);
        }
    });
}
