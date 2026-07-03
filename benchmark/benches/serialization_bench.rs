use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lulu_logs_benchmark::{ProtobufLogWriter, ProtobufLogReader, FlatBuffersLogWriter, FlatBuffersLogReader};
use tempfile::NamedTempFile;
use std::path::Path;

/// Generate test data for benchmarks
fn generate_test_data(count: usize) -> Vec<(String, Vec<u8>, u64)> {
    (0..count)
        .map(|i| {
            (
                format!("lulu/sensor/temperature/{}", i),
                vec![i as u8; 100], // 100 bytes payload
                i as u64 * 1_000_000_000, // timestamp in ns
            )
        })
        .collect()
}

/// Benchmark write performance
fn bench_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_performance");
    
    for count in [100, 1000, 10000].iter() {
        let data = generate_test_data(*count);
        let total_bytes: usize = data.iter().map(|(t, p, _)| t.len() + p.len() + 16).sum();
        
        group.throughput(Throughput::Bytes(total_bytes as u64));
        
        // Protobuf LD
        group.bench_with_input(
            BenchmarkId::new("Protobuf_LD", count),
            &data,
            |b, data| {
                let temp_file = NamedTempFile::new().unwrap();
                let path = temp_file.path();
                b.iter(|| {
                    let mut writer = ProtobufLogWriter::new(path).unwrap();
                    writer.append_batch(black_box(data)).unwrap();
                });
            },
        );
        
        // FlatBuffers Multiples
        group.bench_with_input(
            BenchmarkId::new("FlatBuffers_Multi", count),
            &data,
            |b, data| {
                let temp_file = NamedTempFile::new().unwrap();
                let path = temp_file.path();
                b.iter(|| {
                    let mut writer = FlatBuffersLogWriter::new(path).unwrap();
                    writer.append_batch(black_box(data)).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark read performance (sequential)
fn bench_read_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_sequential");
    
    for count in [100, 1000, 10000].iter() {
        let data = generate_test_data(*count);
        
        group.throughput(Throughput::Bytes((count * 120) as u64)); // ~120 bytes per record
        
        // Protobuf LD
        {
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();
            {
                let mut writer = ProtobufLogWriter::new(path).unwrap();
                writer.append_batch(&data).unwrap();
            }
            
            group.bench_with_input(
                BenchmarkId::new("Protobuf_LD", count),
                &path,
                |b, path| {
                    b.iter(|| {
                        let mut reader = ProtobufLogReader::new(path).unwrap();
                        let mut total_records = 0;
                        while let Some(_) = reader.next().unwrap() {
                            total_records += 1;
                        }
                        black_box(total_records);
                    });
                },
            );
        }
        
        // FlatBuffers Multiples
        {
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();
            {
                let mut writer = FlatBuffersLogWriter::new(path).unwrap();
                writer.append_batch(&data).unwrap();
            }
            
            group.bench_with_input(
                BenchmarkId::new("FlatBuffers_Multi", count),
                &path,
                |b, path| {
                    b.iter(|| {
                        let mut reader = FlatBuffersLogReader::new(path).unwrap();
                        let mut total_records = 0;
                        while let Some(_) = reader.next().unwrap() {
                            total_records += 1;
                        }
                        black_box(total_records);
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark random access (read nth record)
fn bench_random_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_access");
    
    let count = 10000;
    let data = generate_test_data(count);
    
    // Protobuf LD
    {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        {
            let mut writer = ProtobufLogWriter::new(path).unwrap();
            writer.append_batch(&data).unwrap();
        }
        
        group.bench_function("Protobuf_LD_read_5000th", |b| {
            b.iter(|| {
                let mut reader = ProtobufLogReader::new(path).unwrap();
                let record = reader.read_nth(5000).unwrap().unwrap();
                black_box(record);
            });
        });
    }
    
    // FlatBuffers Multiples
    {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        {
            let mut writer = FlatBuffersLogWriter::new(path).unwrap();
            writer.append_batch(&data).unwrap();
        }
        
        group.bench_function("FlatBuffers_Multi_read_5000th", |b| {
            b.iter(|| {
                let mut reader = FlatBuffersLogReader::new(path).unwrap();
                let record = reader.read_nth(5000).unwrap().unwrap();
                black_box(record);
            });
        });
    }
    
    group.finish();
}

/// Benchmark file size comparison
fn bench_file_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_size");
    
    for count in [1000, 10000].iter() {
        let data = generate_test_data(*count);
        
        // Protobuf LD
        {
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();
            {
                let mut writer = ProtobufLogWriter::new(path).unwrap();
                writer.append_batch(&data).unwrap();
            }
            let size = std::fs::metadata(path).unwrap().len();
            group.bench_function(
                BenchmarkId::new("Protobuf_LD", count),
                |b| {
                    b.iter(|| black_box(size));
                },
            );
        }
        
        // FlatBuffers Multiples
        {
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();
            {
                let mut writer = FlatBuffersLogWriter::new(path).unwrap();
                writer.append_batch(&data).unwrap();
            }
            let size = std::fs::metadata(path).unwrap().len();
            group.bench_function(
                BenchmarkId::new("FlatBuffers_Multi", count),
                |b| {
                    b.iter(|| black_box(size));
                },
            );
        }
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_write,
    bench_read_sequential,
    bench_random_access,
    bench_file_size,
);

criterion_main!(benches);
