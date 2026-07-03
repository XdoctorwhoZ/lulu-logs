use flatbuffers::FlatBufferBuilder;
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::Path;

// Include generated FlatBuffers code
use crate::lulu_logs_generated;

/// FlatBuffers LogRecord structure (mirrors the generated type for convenience)
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
        
        // Create the LogRecord using generated code
        let log_record_args = lulu_logs_generated::lulu_logs::LogRecordArgs {
            topic: Some(topic_offset),
            payload: Some(payload_offset),
            timestamp_ns,
        };
        
        let log_record_offset = lulu_logs_generated::lulu_logs::LogRecord::create(&mut builder, &log_record_args);
        
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
        
        // Parse the FlatBuffer using generated code
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
    // Get the root table - the buffer should contain just the flatbuffer data
    // without the length prefix (which was already read by the caller)
    let root = flatbuffers::root::<lulu_logs_generated::lulu_logs::LogRecord>(buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid root: {:?}", e)))?;
    
    let topic = root.topic()
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    let payload = root.payload()
        .map(|v| v.iter().collect::<Vec<u8>>())
        .unwrap_or_default();
    
    let timestamp_ns = root.timestamp_ns();
    
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
