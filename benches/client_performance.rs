//! Benchmarks for LSP client performance
//!
//! This benchmark suite tests the performance of client operations,
//! including message handling, async operations, and throughput.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_lsp::Client;
use std::io::Cursor;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark client creation performance
fn bench_client_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("client_creation");

    group.bench_function("create_client_in_memory", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(Vec::new());
                let writer = Cursor::new(Vec::new());
                let _client = Client::new(black_box(reader), black_box(writer));
            })
        });
    });

    group.bench_function("create_client_with_data", |b| {
        let test_data =
            b"Content-Length: 45\n\n{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}".repeat(10);

        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(test_data.clone()));
                let writer = Cursor::new(Vec::new());
                let _client = Client::new(reader, writer);
            })
        });
    });

    group.finish();
}

/// Benchmark message receiving performance
fn bench_message_receiving(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("message_receiving");

    // Single message - proper server-to-client request
    let single_message = "Content-Length: 91\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"client/registerCapability\",\"params\":{\"registrations\":[]}}";
    group.bench_function("receive_single_message", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(black_box(single_message).as_bytes().to_vec());
                let writer = Cursor::new(Vec::new());
                let mut client = Client::new(reader, writer);
                
                // This should receive the message without timeout
                let _message = client.receive_message().await;
            })
        });
    });

    // Multiple messages
    group.bench_function("receive_multiple_messages", |b| {
        b.iter(|| {
            rt.block_on(async {
                let multiple_messages = create_multiple_messages(10);
                let reader = Cursor::new(black_box(&multiple_messages).as_bytes().to_vec());
                let writer = Cursor::new(Vec::new());
                let mut client = Client::new(reader, writer);

                for _ in 0..10 {
                    let _message = client.receive_message().await;
                }
            })
        });
    });

    group.finish();
}

/// Benchmark client state management
fn bench_client_state(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("client_state");

    group.bench_function("pending_request_count", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(Vec::new());
                let writer = Cursor::new(Vec::new());
                let client = Client::new(reader, writer);
                let _count = black_box(&client).pending_request_count().await;
            })
        });
    });

    group.bench_function("has_pending_requests", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(Vec::new());
                let writer = Cursor::new(Vec::new());
                let client = Client::new(reader, writer);
                let _has_pending = black_box(&client).has_pending_requests().await;
            })
        });
    });

    group.bench_function("cancel_all_requests", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(Vec::new());
                let writer = Cursor::new(Vec::new());
                let client = Client::new(reader, writer);
                black_box(&client).cancel_all_requests().await;
            })
        });
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_operations");

    group.bench_function("concurrent_state_checks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reader = Cursor::new(Vec::new());
                let writer = Cursor::new(Vec::new());
                let client = Arc::new(Client::new(reader, writer));
                
                let tasks: Vec<_> = (0..10)
                    .map(|_| {
                        let client = Arc::clone(&client);
                        tokio::spawn(async move { client.pending_request_count().await })
                    })
                    .collect();

                for task in tasks {
                    let _result = task.await;
                }
            })
        });
    });

    group.finish();
}

/// Benchmark throughput with different message sizes
fn bench_message_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("message_throughput");

    let message_counts = vec![1, 10, 50, 100];

    for count in message_counts {
        group.bench_with_input(
            BenchmarkId::new("small_messages", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let messages = create_small_messages(count);
                        let reader = Cursor::new(black_box(&messages).as_bytes().to_vec());
                        let writer = Cursor::new(Vec::new());
                        let mut client = Client::new(reader, writer);

                        for _ in 0..count {
                            let _message = client.receive_message().await;
                        }
                    })
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("large_messages", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let messages = create_large_messages(count);
                        let reader = Cursor::new(black_box(&messages).as_bytes().to_vec());
                        let writer = Cursor::new(Vec::new());
                        let mut client = Client::new(reader, writer);

                        for _ in 0..count {
                            let _message = client.receive_message().await;
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_patterns");

    // Test with reused data
    group.bench_function("reused_data", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reusable_data = create_multiple_messages(50);
                let reader = Cursor::new(black_box(&reusable_data).as_bytes().to_vec());
                let writer = Cursor::new(Vec::new());
                let mut client = Client::new(reader, writer);

                for _ in 0..50 {
                    let _message = client.receive_message().await;
                }
            })
        });
    });

    // Test with fresh data each time
    group.bench_function("fresh_data", |b| {
        b.iter(|| {
            rt.block_on(async {
                let fresh_data = create_multiple_messages(50);
                let reader = Cursor::new(black_box(&fresh_data).as_bytes().to_vec());
                let writer = Cursor::new(Vec::new());
                let mut client = Client::new(reader, writer);

                for _ in 0..50 {
                    let _message = client.receive_message().await;
                }
            })
        });
    });

    group.finish();
}

// Helper functions to create test data

fn create_multiple_messages(count: usize) -> String {
    let mut result = String::new();
    for i in 0..count {
        // Alternate between different message types
        let message = match i % 3 {
            0 => "Content-Length: 91\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"client/registerCapability\",\"params\":{\"registrations\":[]}}".to_string(),
            1 => "Content-Length: 244\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\"params\":{\"uri\":\"file:///test.rs\",\"diagnostics\":[{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":5}},\"message\":\"test diagnostic\",\"severity\":1,\"source\":\"test\"}]}}".to_string(),
            _ => "Content-Length: 115\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"$/progress\",\"params\":{\"token\":\"workDone\",\"value\":{\"kind\":\"begin\",\"title\":\"Processing\"}}}".to_string(),
        };
        result.push_str(&message);
    }
    result
}

fn create_small_messages(count: usize) -> String {
    let mut result = String::new();
    for _i in 0..count {
        // Simple progress notification
        let message = "Content-Length: 115\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"$/progress\",\"params\":{\"token\":\"workDone\",\"value\":{\"kind\":\"begin\",\"title\":\"Processing\"}}}".to_string();
        result.push_str(&message);
    }
    result
}

fn create_large_messages(count: usize) -> String {
    let mut result = String::new();
    for _i in 0..count {
        // Large diagnostic notification with multiple diagnostics
        let message = "Content-Length: 244\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\"params\":{\"uri\":\"file:///test.rs\",\"diagnostics\":[{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":5}},\"message\":\"test diagnostic\",\"severity\":1,\"source\":\"test\"}]}}".to_string();
        result.push_str(&message);
    }
    result
}


criterion_group!(
    benches,
    bench_client_creation,
    bench_message_receiving,
    bench_client_state,
    bench_concurrent_operations,
    bench_message_throughput,
    bench_memory_patterns
);
criterion_main!(benches);
