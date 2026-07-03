use flatbuffers::{FlatBufferBuilder, WIPOffset};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::Path;

// Manually generated FlatBuffers types (equivalent to what flatc would generate)
// This matches the schema in benchmark/flatbuffers/lulu_logs.fbs

// Table offsets
const LOG_RECORD_VTABLE: [u8; 8] = [8, 0, 8, 0, 12, 0, 0, 0]; // Simplified vtable

/// FlatBuffers LogRecord structure
#[derive(Debug, Clone, PartialEq)]
pub struct FlatBuffersLogRecord {
    pub topic: String,
    pub payload: Vec<u8>,
    pub timestamp_ns: u64,
}

/// Writer for FlatBuffers with length prefix
/// Format: [u32 length][flatbuffer data]
pub struct FlatBuffersLogWriter {
    file: File,
}

impl FlatBuffersLogWriter {
    /// Create a new writer, appending to the file
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }

    /// Append a log record to the file
    pub fn append(&mut self, topic: &str, payload: &[u8], timestamp_ns: u64) -> std::io::Result<()> {
        let mut builder = FlatBufferBuilder::new();
        
        // Create string offset for topic
        let topic_offset = builder.create_string(topic);
        
        // Create payload vector
        let payload_offset = builder.create_vector(payload);
        
        // Create the LogRecord table
        let log_record_offset = self::create_log_record(
            &mut builder,
            topic_offset,
            payload_offset,
            timestamp_ns,
        );
        
        // Finish the buffer
        builder.finish(log_record_offset, None);
        
        let buf = builder.finished_data();
        
        // Write u32 length prefix (big-endian for consistency)
        let length = buf.len() as u32;
        self.file.write_all(&length.to_be_bytes())?;
        
        // Write the flatbuffer data
        self.file.write_all(buf)?;
        self.file.flush()?;
        
        Ok(())
    }

    /// Append multiple records (for batch testing)
    pub fn append_batch(&mut self, records: &[(String, Vec<u8>, u64)]) -> std::io::Result<()> {
        for (topic, payload, timestamp_ns) in records {
            self.append(topic, payload, *timestamp_ns)?;
        }
        Ok(())
    }
}

/// Helper function to create a LogRecord in FlatBuffers
fn create_log_record(
    builder: &mut FlatBufferBuilder,
    topic: WIPOffset<String>,
    payload: WIPOffset<Vec<u8>>,
    timestamp_ns: u64,
) -> WIPOffset<()> {
    // In FlatBuffers, we need to build the table with offsets
    // This is a simplified version of what flatc would generate
    
    // Start the table
    let mut table = builder.start_table();
    
    // Add fields (in reverse order for FlatBuffers)
    table.add(2, timestamp_ns, 0); // timestamp_ns at offset 2
    table.add(1, payload, 0);      // payload at offset 1
    table.add(0, topic, 0);        // topic at offset 0
    
    table.finish()
}

/// Reader for FlatBuffers with length prefix
pub struct FlatBuffersLogReader {
    file: File,
}

impl FlatBuffersLogReader {
    /// Create a new reader
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self { file })
    }

    /// Read the next log record
    pub fn next(&mut self) -> std::io::Result<Option<FlatBuffersLogRecord>> {
        // Read u32 length prefix (big-endian)
        let mut length_bytes = [0u8; 4];
        match self.file.read_exact(&mut length_bytes) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        };
        let length = u32::from_be_bytes(length_bytes) as usize;
        
        // Read flatbuffer data
        let mut buf = vec![0u8; length];
        self.file.read_exact(&mut buf)?;
        
        // Parse the FlatBuffer
        let record = self::read_log_record(&buf)?;
        
        Ok(Some(record))
    }

    /// Read the nth record (0-indexed)
    pub fn read_nth(&mut self, n: usize) -> std::io::Result<Option<FlatBuffersLogRecord>> {
        let mut current = 0;
        loop {
            let record = self.next()?;
            match record {
                Some(rec) => {
                    if current == n {
                        return Ok(Some(rec));
                    }
                    current += 1;
                }
                None => return Ok(None),
            }
        }
    }

    /// Reset to the beginning of the file
    pub fn reset(&mut self) -> std::io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        Ok(())
    }
}

/// Parse a LogRecord from FlatBuffer data
fn read_log_record(buf: &[u8]) -> std::io::Result<FlatBuffersLogRecord> {
    // FlatBuffers uses a buffer with a vtable at the start
    // This is a simplified parser for our specific schema
    
    if buf.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Buffer too short",
        ));
    }
    
    // Get the root table offset (last 4 bytes in little-endian)
    let root_offset = u32::from_le_bytes([buf[buf.len()-4], buf[buf.len()-3], buf[buf.len()-2], buf[buf.len()-1]]) as usize;
    
    if root_offset >= buf.len() - 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid root offset",
        ));
    }
    
    // Parse the table at root_offset
    // The table format: [vtable_offset (2 bytes), field0, field1, ...]
    let vtable_offset = u16::from_le_bytes([buf[root_offset], buf[root_offset+1]]) as usize;
    
    // Get the vtable (contains offsets for each field)
    let vtable_start = root_offset - vtable_offset;
    
    // For our schema:
    // - Field 0: topic (string)
    // - Field 1: payload (vector)
    // - Field 2: timestamp_ns (ulong)
    
    // Read field offsets from vtable
    // Each field in vtable is 2 bytes (offset from table start)
    let topic_field_offset = u16::from_le_bytes([buf[vtable_start], buf[vtable_start+1]]) as usize;
    let payload_field_offset = u16::from_le_bytes([buf[vtable_start+2], buf[vtable_start+3]]) as usize;
    let timestamp_field_offset = u16::from_le_bytes([buf[vtable_start+4], buf[vtable_start+5]]) as usize;
    
    // Get the actual offsets in the buffer
    let topic_offset = root_offset + topic_field_offset;
    let payload_offset = root_offset + payload_field_offset;
    let timestamp_offset = root_offset + timestamp_field_offset;
    
    // Read timestamp (8 bytes, little-endian)
    let timestamp_ns = u64::from_le_bytes([
        buf[timestamp_offset],
        buf[timestamp_offset+1],
        buf[timestamp_offset+2],
        buf[timestamp_offset+3],
        buf[timestamp_offset+4],
        buf[timestamp_offset+5],
        buf[timestamp_offset+6],
        buf[timestamp_offset+7],
    ]);
    
    // Read topic (string offset + length + data)
    let topic_str_offset = u32::from_le_bytes([
        buf[topic_offset],
        buf[topic_offset+1],
        buf[topic_offset+2],
        buf[topic_offset+3],
    ]) as usize;
    let topic_len = u32::from_le_bytes([
        buf[topic_offset+4],
        buf[topic_offset+5],
        buf[topic_offset+6],
        buf[topic_offset+7],
    ]) as usize;
    let topic_start = topic_str_offset;
    let topic = String::from_utf8(buf[topic_start..topic_start+topic_len].to_vec())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    // Read payload (vector offset + length + data)
    let payload_vec_offset = u32::from_le_bytes([
        buf[payload_offset],
        buf[payload_offset+1],
        buf[payload_offset+2],
        buf[payload_offset+3],
    ]) as usize;
    let payload_len = u32::from_le_bytes([
        buf[payload_offset+4],
        buf[payload_offset+5],
        buf[payload_offset+6],
        buf[payload_offset+7],
    ]) as usize;
    let payload_start = payload_vec_offset;
    let payload = buf[payload_start..payload_start+payload_len].to_vec();
    
    Ok(FlatBuffersLogRecord {
        topic,
        payload,
        timestamp_ns,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_roundtrip() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        // Write some records
        {
            let mut writer = FlatBuffersLogWriter::new(path).unwrap();
            writer.append("lulu/sensor/temp", b"data1", 1000).unwrap();
            writer.append("lulu/sensor/humidity", b"data2", 2000).unwrap();
        }
        
        // Read them back
        let mut reader = FlatBuffersLogReader::new(path).unwrap();
        
        let rec1 = reader.next().unwrap().unwrap();
        assert_eq!(rec1.topic, "lulu/sensor/temp");
        assert_eq!(rec1.payload, b"data1");
        assert_eq!(rec1.timestamp_ns, 1000);
        
        let rec2 = reader.next().unwrap().unwrap();
        assert_eq!(rec2.topic, "lulu/sensor/humidity");
        assert_eq!(rec2.payload, b"data2");
        assert_eq!(rec2.timestamp_ns, 2000);
        
        // EOF
        assert!(reader.next().unwrap().is_none());
    }

    #[test]
    fn test_read_nth() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        // Write 10 records
        {
            let mut writer = FlatBuffersLogWriter::new(path).unwrap();
            for i in 0..10 {
                writer.append(&format!("lulu/sensor/{}", i), &[i as u8], i as u64).unwrap();
            }
        }
        
        // Read the 5th record (0-indexed)
        let mut reader = FlatBuffersLogReader::new(path).unwrap();
        let rec = reader.read_nth(5).unwrap().unwrap();
        assert_eq!(rec.topic, "lulu/sensor/5");
        assert_eq!(rec.payload, &[5u8]);
        assert_eq!(rec.timestamp_ns, 5);
    }
}
