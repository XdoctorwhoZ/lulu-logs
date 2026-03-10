use std::fmt;
use std::str::FromStr;

use crate::generated::lulu_logs_generated::lulu_logs::LogLevel as FbsLogLevel;

// ---------------------------------------------------------------------------
// LuluLevel
// ---------------------------------------------------------------------------

/// Severity level for a lulu-logs entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LuluLevel {
    Trace = 0,
    Debug = 1,
    Info  = 2,
    Warn  = 3,
    Error = 4,
    Fatal = 5,
}

impl LuluLevel {
    /// Returns the CSS class name associated with this level.
    pub fn css_class(&self) -> &'static str {
        match self {
            LuluLevel::Trace => "level-trace",
            LuluLevel::Debug => "level-debug",
            LuluLevel::Info  => "level-info",
            LuluLevel::Warn  => "level-warn",
            LuluLevel::Error => "level-error",
            LuluLevel::Fatal => "level-fatal",
        }
    }

    /// Converts from the FlatBuffers-generated `LogLevel`.
    pub fn from_fbs(fbs: FbsLogLevel) -> Self {
        match fbs {
            FbsLogLevel::Trace => LuluLevel::Trace,
            FbsLogLevel::Debug => LuluLevel::Debug,
            FbsLogLevel::Info  => LuluLevel::Info,
            FbsLogLevel::Warn  => LuluLevel::Warn,
            FbsLogLevel::Error => LuluLevel::Error,
            FbsLogLevel::Fatal => LuluLevel::Fatal,
            _ => LuluLevel::Info,
        }
    }
}

impl fmt::Display for LuluLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuluLevel::Trace => write!(f, "Trace"),
            LuluLevel::Debug => write!(f, "Debug"),
            LuluLevel::Info  => write!(f, "Info"),
            LuluLevel::Warn  => write!(f, "Warn"),
            LuluLevel::Error => write!(f, "Error"),
            LuluLevel::Fatal => write!(f, "Fatal"),
        }
    }
}

impl FromStr for LuluLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LuluLevel::Trace),
            "debug" => Ok(LuluLevel::Debug),
            "info"  => Ok(LuluLevel::Info),
            "warn"  => Ok(LuluLevel::Warn),
            "error" => Ok(LuluLevel::Error),
            "fatal" => Ok(LuluLevel::Fatal),
            _ => Err(format!("unknown log level: {}", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// LuluLogEntry
// ---------------------------------------------------------------------------

/// A log entry as stored in memory by the application.
///
/// Contains data extracted from the MQTT topic and FlatBuffers payload,
/// plus the raw payload for export.
#[derive(Debug, Clone, PartialEq)]
pub struct LuluLogEntry {
    /// Full MQTT topic received (e.g. "lulu/psu/power-supply/channel-1/voltage").
    pub topic: String,

    /// Multi-level source extracted from the topic (segments 1..N-1 rejoined with "/").
    /// Example: "psu/power-supply/channel-1"
    pub source: String,

    /// Attribute extracted from the topic (last segment).
    /// Example: "voltage"
    pub attribute: String,

    /// ISO 8601 UTC timestamp as received in the payload.
    pub timestamp: String,

    /// Severity level.
    pub level: LuluLevel,

    /// Data type descriptor (cf. lulu-logs spec § 3.3).
    pub data_type: String,

    /// Decoded value ready for display (cf. lulu-logs spec § 3.3).
    pub decoded_value: String,

    /// Raw data bytes extracted from the FlatBuffers payload (pre-interpretation).
    pub data_bytes: Vec<u8>,

    /// Raw FlatBuffers payload (kept for export).
    pub raw_payload: Vec<u8>,
}

// ---------------------------------------------------------------------------
// decode_data
// ---------------------------------------------------------------------------

/// Decodes the raw bytes `data` according to the type descriptor `type_str`.
///
/// Returns a display-ready string, or `"[decode error: …]"` on failure.
pub fn decode_data(type_str: &str, data: &[u8]) -> String {
    match type_str {
        "string" => decode_string(data),
        "int32"  => decode_int32(data),
        "int64"  => decode_int64(data),
        "float32" => decode_float32(data),
        "float64" => decode_float64(data),
        "bool"   => decode_bool(data),
        "json"   => decode_json(data),
        "bytes"  => decode_bytes(data),
        "net_packet" | "serial_chunk" => decode_bytes(data),
        "beg_test_scenario" | "end_test_scenario" => decode_json(data),
        _ => format!("[decode error: unknown type \"{}\"]", type_str),
    }
}

fn decode_string(data: &[u8]) -> String {
    match std::str::from_utf8(data) {
        Ok(s) => s.to_string(),
        Err(e) => format!("[decode error: invalid UTF-8: {}]", e),
    }
}

fn decode_int32(data: &[u8]) -> String {
    if data.len() != 4 {
        return format!("[decode error: expected 4 bytes for int32, got {}]", data.len());
    }
    let val = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    val.to_string()
}

fn decode_int64(data: &[u8]) -> String {
    if data.len() != 8 {
        return format!("[decode error: expected 8 bytes for int64, got {}]", data.len());
    }
    let val = i64::from_le_bytes([
        data[0], data[1], data[2], data[3],
        data[4], data[5], data[6], data[7],
    ]);
    val.to_string()
}

fn decode_float32(data: &[u8]) -> String {
    if data.len() != 4 {
        return format!("[decode error: expected 4 bytes for float32, got {}]", data.len());
    }
    let val = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    format!("{:.6}", val)
}

fn decode_float64(data: &[u8]) -> String {
    if data.len() != 8 {
        return format!("[decode error: expected 8 bytes for float64, got {}]", data.len());
    }
    let val = f64::from_le_bytes([
        data[0], data[1], data[2], data[3],
        data[4], data[5], data[6], data[7],
    ]);
    format!("{:.10}", val)
}

fn decode_bool(data: &[u8]) -> String {
    if data.len() != 1 {
        return format!("[decode error: expected 1 byte for bool, got {}]", data.len());
    }
    match data[0] {
        0x00 => "false".to_string(),
        0x01 => "true".to_string(),
        v => format!("[decode error: unexpected bool value 0x{:02X}]", v),
    }
}

fn decode_json(data: &[u8]) -> String {
    match std::str::from_utf8(data) {
        Ok(s) => {
            // Attempt to pretty-print the JSON
            match serde_json::from_str::<serde_json::Value>(s) {
                Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|_| s.to_string()),
                Err(_) => s.to_string(),
            }
        }
        Err(e) => format!("[decode error: invalid UTF-8 for json: {}]", e),
    }
}

fn decode_bytes(data: &[u8]) -> String {
    let max_display = 64;
    let to_show = if data.len() > max_display { &data[..max_display] } else { data };
    let hex: Vec<String> = to_show.iter().map(|b| format!("0x{:02X}", b)).collect();
    let mut result = hex.join(" ");
    if data.len() > max_display {
        result.push_str(&format!(" … ({} bytes total)", data.len()));
    }
    result
}
