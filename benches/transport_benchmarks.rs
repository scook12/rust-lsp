//! Benchmarks for transport layer performance
//!
//! This benchmark suite tests the performance of different transport
//! implementations, async I/O operations, and message framing.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tokio_lsp::{transport::*, types::*};
use std::io::Cursor;
use tokio::runtime::Runtime;

/// Benchmark transport creation with different data sizes
fn bench_transport_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport_creation");

    let data_sizes = vec![("small", 100), ("medium", 10_000), ("large", 100_000)];

    for (size_name, size) in data_sizes {
        group.bench_with_input(
            BenchmarkId::new("create_transport", size_name),
            &size,
            |b, &size| {
                let data = vec![0u8; size];
                b.iter(|| {
                    let reader = Cursor::new(black_box(&data));
                    let writer = Cursor::new(Vec::new());
                    let _transport = Transport::new(reader, writer);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark message reading performance
fn bench_message_reading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("message_reading");

    // Simple message
    let simple_message = "Content-Length: 35\n\n{\"jsonrpc\":\"2.0\",\"method\":\"test\"}";
    group.bench_function("read_simple_message", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(simple_message).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _message = transport.read_message().await;
            })
        });
    });

    // Complex message with large payload
    let complex_data = "x".repeat(5000);
    let complex_message = format!(
        "Content-Length: {}\n\n{{\"jsonrpc\":\"2.0\",\"method\":\"large\",\"params\":{{\"data\":\"{}\"}}}}",
        60 + complex_data.len(),
        complex_data
    );
    group.bench_function("read_complex_message", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(&complex_message).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _message = transport.read_message().await;
            })
        });
    });

    // Multiple messages in sequence
    let multiple_messages = create_message_sequence(20);
    group.bench_function("read_message_sequence", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(&multiple_messages).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                for _ in 0..20 {
                    let _message = transport.read_message().await;
                }
            })
        });
    });

    group.finish();
}

/// Benchmark RPC message parsing performance
fn bench_rpc_parsing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("rpc_parsing");

    let message_types = vec![
        ("request", create_request_message()),
        ("response", create_response_message()),
        ("notification", create_notification_message()),
        ("error_response", create_error_response_message()),
    ];

    for (msg_type, raw_message) in message_types {
        group.bench_with_input(
            BenchmarkId::new("parse_rpc_message", msg_type),
            &raw_message,
            |b, message| {
                b.iter(|| {
                    rt.block_on(async {
                        let reader = Cursor::new(black_box(message).as_bytes());
                        let writer = Cursor::new(Vec::new());
                        let mut transport = Transport::new(reader, writer);

                        if let Ok(message) = transport.read_message().await {
                            let _rpc = message.parse_rpc_message();
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark message writing performance
fn bench_message_writing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("message_writing");

    // Create different types of RPC messages
    let request = RpcMessage::Request(RequestMessage::new(Id::Number(1), "test/method"));
    let notification = RpcMessage::Notification(NotificationMessage::new("test/notification"));
    let response = RpcMessage::Response(ResponseMessage::success(
        Id::Number(1),
        serde_json::json!({"result": "success"}),
    ));

    group.bench_function("write_request", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(Vec::new());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _result = transport.write_rpc_message(black_box(&request)).await;
            })
        });
    });

    group.bench_function("write_notification", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(Vec::new());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _result = transport.write_rpc_message(black_box(&notification)).await;
            })
        });
    });

    group.bench_function("write_response", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(Vec::new());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _result = transport.write_rpc_message(black_box(&response)).await;
            })
        });
    });

    group.finish();
}

/// Benchmark different AsyncRead/AsyncWrite implementations
fn bench_different_transports(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("transport_types");

    let test_message = "Content-Length: 35\n\n{\"jsonrpc\":\"2.0\",\"method\":\"test\"}";

    // Cursor<Vec<u8>>
    group.bench_function("cursor_vec", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(test_message).as_bytes().to_vec());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _message = transport.read_message().await;
            })
        });
    });

    // Cursor<&[u8]>
    group.bench_function("cursor_slice", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(test_message).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _message = transport.read_message().await;
            })
        });
    });

    group.finish();
}

/// Benchmark message throughput under load
fn bench_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("throughput");

    let message_counts = vec![10, 50, 100, 500];

    for count in message_counts {
        group.bench_with_input(
            BenchmarkId::new("messages_per_second", count),
            &count,
            |b, &count| {
                let messages = create_message_sequence(count);
                b.iter(|| {
                    rt.block_on(async {
                        let reader = Cursor::new(black_box(&messages).as_bytes());
                        let writer = Cursor::new(Vec::new());
                        let mut transport = Transport::new(reader, writer);

                        for _ in 0..count {
                            if let Ok(message) = transport.read_message().await {
                                let _rpc = message.parse_rpc_message();
                            }
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark header parsing performance
fn bench_header_parsing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("header_parsing");

    // Simple header
    let simple_header = "Content-Length: 25\n\n{\"jsonrpc\":\"2.0\",\"id\":1}";
    group.bench_function("simple_header", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(simple_header).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _message = transport.read_message().await;
            })
        });
    });

    // Header with extra fields
    let complex_header =
        "Content-Length: 25\nContent-Type: application/json\n\n{\"jsonrpc\":\"2.0\",\"id\":1}";
    group.bench_function("complex_header", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(complex_header).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _message = transport.read_message().await;
            })
        });
    });

    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("error_handling");

    // Invalid content length
    let invalid_length = "Content-Length: invalid\n\n{\"test\": true}";
    group.bench_function("invalid_content_length", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(invalid_length).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _result = transport.read_message().await;
            })
        });
    });

    // Missing header
    let missing_header = "\n\n{\"test\": true}";
    group.bench_function("missing_header", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(missing_header).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                let _result = transport.read_message().await;
            })
        });
    });

    // Invalid JSON
    let invalid_json = "Content-Length: 15\n\n{invalid json}";
    group.bench_function("invalid_json", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(invalid_json).as_bytes());
                let writer = Cursor::new(Vec::new());
                let mut transport = Transport::new(reader, writer);

                if let Ok(message) = transport.read_message().await {
                    let _result = message.parse_rpc_message();
                }
            })
        });
    });

    group.finish();
}

// Helper functions to create test data

fn create_message_sequence(count: usize) -> String {
    let mut result = String::new();
    for i in 0..count {
        let message_body = format!("{{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":{}}}", i);
        let message = format!("Content-Length: {}\n\n{}", message_body.len(), message_body);
        result.push_str(&message);
    }
    result
}

fn create_request_message() -> String {
    let message_body = r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.rs"},"position":{"line":10,"character":5}}}"#;
    format!("Content-Length: {}\n\n{}", message_body.len(), message_body)
}

fn create_response_message() -> String {
    let message_body = r#"{"jsonrpc":"2.0","id":1,"result":{"contents":{"kind":"markdown","value":"**Function**: test"}}}"#;
    format!("Content-Length: {}\n\n{}", message_body.len(), message_body)
}

fn create_notification_message() -> String {
    let message_body = r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///test.rs","diagnostics":[]}}"#;
    format!("Content-Length: {}\n\n{}", message_body.len(), message_body)
}

fn create_error_response_message() -> String {
    let message_body =
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
    format!("Content-Length: {}\n\n{}", message_body.len(), message_body)
}

criterion_group!(
    benches,
    bench_transport_creation,
    bench_message_reading,
    bench_rpc_parsing,
    bench_message_writing,
    bench_different_transports,
    bench_throughput,
    bench_header_parsing,
    bench_error_handling
);
criterion_main!(benches);
