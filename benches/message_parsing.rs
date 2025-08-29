//! Benchmarks for message parsing performance
//!
//! This benchmark suite tests the performance of JSON-RPC message parsing,
//! serialization, and various LSP type operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_lsp::types::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Benchmark JSON-RPC message parsing performance
fn bench_message_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_parsing");

    // Test different message sizes
    let message_sizes = vec![
        ("small", create_small_message()),
        ("medium", create_medium_message()),
        ("large", create_large_message()),
    ];

    for (size, message) in message_sizes {
        group.bench_with_input(
            BenchmarkId::new("request_parsing", size),
            &message,
            |b, msg| {
                b.iter(|| {
                    let _: RequestMessage = serde_json::from_str(black_box(msg)).unwrap();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("response_parsing", size),
            &message.replace("\"method\":", "\"result\":"),
            |b, msg| {
                b.iter(|| {
                    let _: ResponseMessage = serde_json::from_str(black_box(msg)).unwrap();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("notification_parsing", size),
            &message.replace("\"id\":1,", ""),
            |b, msg| {
                b.iter(|| {
                    let _: NotificationMessage = serde_json::from_str(black_box(msg)).unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark LSP type serialization performance
fn bench_type_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_serialization");

    let position = Position::new(100, 50);
    let range = Range::new(Position::new(10, 0), Position::new(20, 80));
    let diagnostic = create_test_diagnostic();
    let init_params = create_test_init_params();

    group.bench_function("position_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&position)).unwrap();
        });
    });

    group.bench_function("range_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&range)).unwrap();
        });
    });

    group.bench_function("diagnostic_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&diagnostic)).unwrap();
        });
    });

    group.bench_function("init_params_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&init_params)).unwrap();
        });
    });

    group.finish();
}

/// Benchmark RpcMessage enum parsing performance
fn bench_rpc_message_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("rpc_message_parsing");

    let request_json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "textDocument/hover",
        "params": {"textDocument": {"uri": "file:///test.rs"}, "position": {"line": 10, "character": 5}}
    });

    let response_json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"contents": {"kind": "markdown", "value": "**Function**: test"}}
    });

    let notification_json = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": "file:///test.rs", "diagnostics": []}
    });

    group.bench_function("request_message", |b| {
        b.iter(|| {
            let _: RpcMessage = serde_json::from_value(black_box(request_json.clone())).unwrap();
        });
    });

    group.bench_function("response_message", |b| {
        b.iter(|| {
            let _: RpcMessage = serde_json::from_value(black_box(response_json.clone())).unwrap();
        });
    });

    group.bench_function("notification_message", |b| {
        b.iter(|| {
            let _: RpcMessage =
                serde_json::from_value(black_box(notification_json.clone())).unwrap();
        });
    });

    group.finish();
}

/// Benchmark ID type operations
fn bench_id_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_operations");

    let number_id = Id::Number(12345);
    let string_id = Id::String("request-abc-123".to_string());

    group.bench_function("number_id_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&number_id)).unwrap();
        });
    });

    group.bench_function("string_id_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&string_id)).unwrap();
        });
    });

    group.bench_function("number_id_display", |b| {
        b.iter(|| {
            let _s = black_box(&number_id).to_string();
        });
    });

    group.bench_function("string_id_display", |b| {
        b.iter(|| {
            let _s = black_box(&string_id).to_string();
        });
    });

    group.bench_function("id_hash", |b| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            black_box(&number_id).hash(&mut hasher);
            let _hash = hasher.finish();
        });
    });

    group.finish();
}

/// Benchmark complex message parsing
fn bench_complex_messages(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_messages");

    // Large diagnostic message
    let mut diagnostics = Vec::new();
    for i in 0..100 {
        diagnostics.push(json!({
            "range": {
                "start": {"line": i, "character": 0},
                "end": {"line": i, "character": 10}
            },
            "message": format!("Diagnostic message {}", i),
            "severity": (i % 4) + 1,
            "code": format!("E{:04}", i),
            "source": "test-lsp"
        }));
    }

    let large_diagnostic_msg = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": "file:///large-file.rs",
            "diagnostics": diagnostics
        }
    });

    group.bench_function("large_diagnostic_message", |b| {
        b.iter(|| {
            let _: RpcMessage =
                serde_json::from_value(black_box(large_diagnostic_msg.clone())).unwrap();
        });
    });

    // Complex initialization message
    let complex_init = create_complex_init_message();
    group.bench_function("complex_initialization", |b| {
        b.iter(|| {
            let _: InitializeParams =
                serde_json::from_value(black_box(complex_init.clone())).unwrap();
        });
    });

    group.finish();
}

// Helper functions to create test data

fn create_small_message() -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "test",
        "params": {"key": "value"}
    })
    .to_string()
}

fn create_medium_message() -> String {
    let params = json!({
        "textDocument": {"uri": "file:///test.rs"},
        "position": {"line": 10, "character": 5},
        "context": {
            "includeDeclaration": true,
            "triggerKind": 1,
            "triggerCharacter": "."
        },
        "workDoneToken": "progress-token-123"
    });

    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "textDocument/references",
        "params": params
    })
    .to_string()
}

fn create_large_message() -> String {
    let mut large_params = HashMap::new();
    for i in 0..50 {
        large_params.insert(format!("key_{}", i), format!("value_{}", i));
    }

    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "workspace/executeCommand",
        "params": {
            "command": "complex.command",
            "arguments": [large_params],
            "workDoneToken": "work-done-token-xyz-123",
            "metadata": {
                "source": "test-client",
                "timestamp": "2023-01-01T00:00:00Z",
                "data": "x".repeat(1000)
            }
        }
    })
    .to_string()
}

fn create_test_diagnostic() -> Diagnostic {
    Diagnostic {
        range: Range::new(Position::new(5, 10), Position::new(5, 20)),
        severity: Some(DiagnosticSeverity::Error),
        code: Some(DiagnosticCode::String("E0001".to_string())),
        code_description: None,
        source: Some("rust-analyzer".to_string()),
        message: "undefined variable `test_var`".to_string(),
        tags: Some(vec![DiagnosticTag::Unnecessary]),
        related_information: None,
        data: Some(json!({"quickfix": "declare_variable"})),
    }
}

fn create_test_init_params() -> InitializeParams {
    InitializeParams {
        process_id: Some(12345),
        client_info: Some(ClientInfo {
            name: "Test LSP Client".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        root_uri: Some("file:///test/project".to_string()),
        capabilities: ClientCapabilities::default(),
        root_path: None,
        locale: Some("en-US".to_string()),
        initialization_options: Some(json!({"custom": "options"})),
        trace: None,
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: "file:///test/project".to_string(),
            name: "Test Project".to_string(),
        }]),
    }
}

fn create_complex_init_message() -> Value {
    json!({
        "processId": 12345,
        "clientInfo": {"name": "Complex Client", "version": "2.0.0"},
        "rootUri": "file:///complex/project",
        "capabilities": {
            "workspace": {
                "applyEdit": true,
                "workspaceEdit": {
                    "documentChanges": true,
                    "resourceOperations": ["create", "rename", "delete"],
                    "failureHandling": "transactional"
                },
                "didChangeConfiguration": {"dynamicRegistration": true},
                "didChangeWatchedFiles": {"dynamicRegistration": true},
                "symbol": {"dynamicRegistration": true},
                "executeCommand": {"dynamicRegistration": true},
                "workspaceFolders": true,
                "configuration": true
            },
            "textDocument": {
                "synchronization": {
                    "dynamicRegistration": true,
                    "willSave": true,
                    "willSaveWaitUntil": true,
                    "didSave": true
                },
                "completion": {
                    "dynamicRegistration": true,
                    "completionItem": {
                        "snippetSupport": true,
                        "commitCharactersSupport": true
                    }
                },
                "hover": {
                    "dynamicRegistration": true,
                    "contentFormat": ["markdown", "plaintext"]
                }
            }
        },
        "initializationOptions": {
            "complexSettings": {
                "nested": {
                    "deeply": {
                        "configuration": true
                    }
                }
            }
        },
        "workspaceFolders": [
            {"uri": "file:///project1", "name": "Project 1"},
            {"uri": "file:///project2", "name": "Project 2"}
        ]
    })
}

criterion_group!(
    benches,
    bench_message_parsing,
    bench_type_serialization,
    bench_rpc_message_parsing,
    bench_id_operations,
    bench_complex_messages
);
criterion_main!(benches);
