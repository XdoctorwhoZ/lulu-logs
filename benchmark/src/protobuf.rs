use prost::Message;
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::Path;

// Include generated Protobuf code
mod proto {
    include!(concat!(env!("OUT_DIR"), "/lulu_logs.rs"));
}

use proto::LogRecord;

/// Writer for Protobuf Length-Delimited format
/// Format: [varint length][protobuf data]
pub struct ProtobufLogWriter {
    file: File,
}

impl ProtobufLogWriter {
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
        let record = LogRecord {
            topic: topic.to_string(),
            payload: payload.to_vec(),
            timestamp_ns,
        };
        
        let mut buf = Vec::new();
        record.encode(&mut buf)?;
        
        // Write varint length prefix
        prost::encoding::encode_varint(buf.len() as u64, &mut self.file)?;
        
        // Write the protobuf data
        self.file.write_all(&buf)?;
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

/// Reader for Protobuf Length-Delimited format
pub struct ProtobufLogReader {
    file: File,
}

impl ProtobufLogReader {
    /// Create a new reader
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self { file })
    }

    /// Read the next log record
    pub fn next(&mut self) -> std::io::Result<Option<LogRecord>> {
        // Read varint length
        let length = match prost::encoding::decode_varint(&mut self.file) {
            Ok(len) => len as usize,
            Err(_) => return Ok(None), // EOF
        };
        
        // Read protobuf data
        let mut buf = vec![0u8; length];
        self.file.read_exact(&mut buf)?;
        
        // Decode the record
        let record = LogRecord::decode(&buf[..])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        Ok(Some(record))
    }

    /// Read the nth record (0-indexed)
    pub fn read_nth(&mut self, n: usize) -> std::io::Result<Option<LogRecord>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_roundtrip() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        // Write some records
        {
            let mut writer = ProtobufLogWriter::new(path).unwrap();
            writer.append("lulu/sensor/temp", b"data1", 1000).unwrap();
            writer.append("lulu/sensor/humidity", b"data2", 2000).unwrap();
        }
        
        // Read them back
        let mut reader = ProtobufLogReader::new(path).unwrap();
        
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
            let mut writer = ProtobufLogWriter::new(path).unwrap();
            for i in 0..10 {
                writer.append(&format!("lulu/sensor/{}", i), &[i as u8], i as u64).unwrap();
            }
        }
        
        // Read the 5th record (0-indexed)
        let mut reader = ProtobufLogReader::new(path).unwrap();
        let rec = reader.read_nth(5).unwrap().unwrap();
        assert_eq!(rec.topic, "lulu/sensor/5");
        assert_eq!(rec.payload, &[5u8]);
        assert_eq!(rec.timestamp_ns, 5);
    }
}
