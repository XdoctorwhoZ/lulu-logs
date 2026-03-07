# LULU LOGS

Lulu-Logs is a logging system designed to merge heterogeneous test data into a single timeline and produce interactive test reports.

## Application

The application captures and analyzes logs from your testing sequences in real-time.

![](app_capture.png)

Pin some data sources to zoom-in the analysis.

![](app_capture_2.png)

## Rust Client

The Rust client provides a simple singleton API to send logs over MQTT to the Lulu-Logs system.

### Usage

First, initialize the client with your MQTT broker configuration:

```rust
use lulu_logs_client::{lulu_init, lulu_publish, LogLevel, Data, LuluClientConfig};

// Initialize the client
let config = LuluClientConfig {
    broker_host: "127.0.0.1".to_string(),
    broker_port: 1883,
    ..Default::default()
};
lulu_init(config)?;

// Publish a log entry
lulu_publish(
    "device/sensor-1",           // source (hierarchical path)
    "temperature",                // attribute name
    LogLevel::Info,                // log level
    Data::Float(23.5),             // data value
)?;

// Gracefully shutdown when done
lulu_shutdown();
```


