//! LSP initialization types and structures.
//!
//! This module contains types related to the initialization handshake
//! between client and server as defined by the LSP specification.

use crate::types::DocumentUri;
use serde::{Deserialize, Serialize};

/// Capabilities that the client supports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Workspace-specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceClientCapabilities>,

    /// Text document-specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document: Option<TextDocumentClientCapabilities>,

    /// Capabilities specific to the notebook document support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notebook_document: Option<NotebookDocumentClientCapabilities>,

    /// Window-specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowClientCapabilities>,

    /// General client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub general: Option<GeneralClientCapabilities>,

    /// Experimental client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<serde_json::Value>,
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self {
            workspace: Some(WorkspaceClientCapabilities::default()),
            text_document: Some(TextDocumentClientCapabilities::default()),
            notebook_document: None,
            window: Some(WindowClientCapabilities::default()),
            general: Some(GeneralClientCapabilities::default()),
            experimental: None,
        }
    }
}

/// Workspace-specific client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceClientCapabilities {
    /// The client supports applying batch edits to the workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_edit: Option<bool>,

    /// Capabilities specific to `WorkspaceEdit`s.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_edit: Option<WorkspaceEditClientCapabilities>,

    /// Capabilities specific to the `workspace/didChangeConfiguration` notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_configuration: Option<DidChangeConfigurationClientCapabilities>,

    /// Capabilities specific to the `workspace/didChangeWatchedFiles` notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_watched_files: Option<DidChangeWatchedFilesClientCapabilities>,

    /// Capabilities specific to the `workspace/symbol` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<WorkspaceSymbolClientCapabilities>,

    /// Capabilities specific to the `workspace/executeCommand` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_command: Option<ExecuteCommandClientCapabilities>,

    /// The client has support for workspace folders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<bool>,

    /// The client supports `workspace/configuration` requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<bool>,
}

impl Default for WorkspaceClientCapabilities {
    fn default() -> Self {
        Self {
            apply_edit: Some(true),
            workspace_edit: Some(WorkspaceEditClientCapabilities::default()),
            did_change_configuration: Some(DidChangeConfigurationClientCapabilities {
                dynamic_registration: Some(false),
            }),
            did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                dynamic_registration: Some(false),
            }),
            symbol: None,
            execute_command: Some(ExecuteCommandClientCapabilities {
                dynamic_registration: Some(false),
            }),
            workspace_folders: Some(true),
            configuration: Some(true),
        }
    }
}

/// Client capabilities for workspace edit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceEditClientCapabilities {
    /// The client supports versioned document changes in `WorkspaceEdit`s.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_changes: Option<bool>,

    /// The resource operations the client supports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_operations: Option<Vec<ResourceOperationKind>>,

    /// The failure handling strategy of a client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_handling: Option<FailureHandlingKind>,

    /// Whether the client normalizes line endings to the client specific setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalizes_line_endings: Option<bool>,

    /// Whether the client in general supports change annotations on text edits,
    /// create file, rename file and delete file changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_annotation_support: Option<ChangeAnnotationsSupportOptions>,
}

impl Default for WorkspaceEditClientCapabilities {
    fn default() -> Self {
        Self {
            document_changes: Some(true),
            resource_operations: Some(vec![
                ResourceOperationKind::Create,
                ResourceOperationKind::Rename,
                ResourceOperationKind::Delete,
            ]),
            failure_handling: Some(FailureHandlingKind::Transactional),
            normalizes_line_endings: Some(false),
            change_annotation_support: None,
        }
    }
}

/// The kind of resource operations supported by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceOperationKind {
    /// Supports creating new files and folders.
    Create,
    /// Supports renaming existing files and folders.
    Rename,
    /// Supports deleting existing files and folders.
    Delete,
}

/// The failure handling strategy for workspace edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureHandlingKind {
    /// Applying the workspace change is simply aborted if one of the changes
    /// provided fails. All operations executed before the failing operation
    /// stay executed.
    Abort,
    /// All operations are executed transactionally. That means they either all
    /// succeed or no changes at all are applied to the workspace.
    Transactional,
    /// If the workspace edit contains only textual file changes they are executed
    /// transactionally. If resource changes (create, rename or delete file) are part
    /// of the change the failure handling strategy is abort.
    TextOnlyTransactional,
    /// The client tries to undo the operations already executed. But there is no
    /// guarantee that this is succeeding.
    Undo,
}

/// Change annotations support options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeAnnotationsSupportOptions {
    /// Whether the client groups edits with equal labels into tree nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups_on_label: Option<bool>,
}

/// Simple client capabilities with dynamic registration support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DidChangeConfigurationClientCapabilities {
    /// Did change configuration notification supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,
}

/// Simple client capabilities with dynamic registration support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DidChangeWatchedFilesClientCapabilities {
    /// Did change watched files notification supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,
}

/// Client capabilities for workspace symbol requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSymbolClientCapabilities {
    /// Symbol request supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,
}

/// Client capabilities for execute command requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecuteCommandClientCapabilities {
    /// Execute command supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,
}

/// Text document-specific client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentClientCapabilities {
    /// Defines which synchronization capabilities the client supports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronization: Option<TextDocumentSyncClientCapabilities>,

    /// Capabilities specific to the `textDocument/completion` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<CompletionClientCapabilities>,

    /// Capabilities specific to the `textDocument/hover` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<HoverClientCapabilities>,
}

impl Default for TextDocumentClientCapabilities {
    fn default() -> Self {
        Self {
            synchronization: Some(TextDocumentSyncClientCapabilities::default()),
            completion: Some(CompletionClientCapabilities::default()),
            hover: Some(HoverClientCapabilities::default()),
        }
    }
}

/// Text document synchronization client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentSyncClientCapabilities {
    /// Whether text document synchronization supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client supports sending will save notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save: Option<bool>,

    /// The client supports sending a will save request and waits for a response
    /// providing text edits which will be applied to the document before it is saved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save_wait_until: Option<bool>,

    /// The client supports did save notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_save: Option<bool>,
}

impl Default for TextDocumentSyncClientCapabilities {
    fn default() -> Self {
        Self {
            dynamic_registration: Some(false),
            will_save: Some(true),
            will_save_wait_until: Some(true),
            did_save: Some(true),
        }
    }
}

/// Completion client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionClientCapabilities {
    /// Whether completion supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client supports the following `CompletionItem` specific capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_item: Option<CompletionItemClientCapabilities>,
}

impl Default for CompletionClientCapabilities {
    fn default() -> Self {
        Self {
            dynamic_registration: Some(false),
            completion_item: Some(CompletionItemClientCapabilities::default()),
        }
    }
}

/// Completion item client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionItemClientCapabilities {
    /// Client supports snippets as insert text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet_support: Option<bool>,

    /// Client supports commit characters on a completion item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_characters_support: Option<bool>,

    /// Client supports the following content formats for the documentation
    /// property. The order describes the preferred format of the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_format: Option<Vec<MarkupKind>>,
}

impl Default for CompletionItemClientCapabilities {
    fn default() -> Self {
        Self {
            snippet_support: Some(true),
            commit_characters_support: Some(true),
            documentation_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
        }
    }
}

/// Hover client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoverClientCapabilities {
    /// Whether hover supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Client supports the following content formats for the content
    /// property. The order describes the preferred format of the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_format: Option<Vec<MarkupKind>>,
}

impl Default for HoverClientCapabilities {
    fn default() -> Self {
        Self {
            dynamic_registration: Some(false),
            content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
        }
    }
}

/// Describes the content type that a client supports in various result types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkupKind {
    /// Plain text is supported as a content format.
    PlainText,
    /// Markdown is supported as a content format.
    Markdown,
}

/// Notebook document client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotebookDocumentClientCapabilities {
    /// Capabilities specific to notebook document synchronization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronization: Option<NotebookDocumentSyncClientCapabilities>,
}

/// Notebook document synchronization client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotebookDocumentSyncClientCapabilities {
    /// Whether implementation supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client supports sending execution summary data per cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_summary_support: Option<bool>,
}

/// Window specific client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowClientCapabilities {
    /// It indicates whether the client supports server initiated progress using the
    /// `window/workDoneProgress/create` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_done_progress: Option<bool>,

    /// Capabilities specific to the showMessage request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_message: Option<ShowMessageRequestClientCapabilities>,

    /// Capabilities specific to the showDocument request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_document: Option<ShowDocumentClientCapabilities>,
}

impl Default for WindowClientCapabilities {
    fn default() -> Self {
        Self {
            work_done_progress: Some(true),
            show_message: None,
            show_document: None,
        }
    }
}

/// Show message request client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShowMessageRequestClientCapabilities {
    /// Capabilities specific to the `MessageActionItem` type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_action_item: Option<MessageActionItemClientCapabilities>,
}

/// Message action item client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageActionItemClientCapabilities {
    /// Whether the client supports additional attributes which
    /// are preserved and send back to the server in the
    /// request's response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties_support: Option<bool>,
}

/// Show document client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShowDocumentClientCapabilities {
    /// The client has support for the showDocument request.
    pub support: bool,
}

/// General client capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralClientCapabilities {
    /// Client capabilities specific to regular expressions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_expressions: Option<RegularExpressionsClientCapabilities>,

    /// Client capabilities specific to the client's markdown parser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<MarkdownClientCapabilities>,
}

impl Default for GeneralClientCapabilities {
    fn default() -> Self {
        Self {
            regular_expressions: Some(RegularExpressionsClientCapabilities {
                engine: "ECMAScript".to_string(),
                version: Some("ES2020".to_string()),
            }),
            markdown: Some(MarkdownClientCapabilities {
                parser: "marked".to_string(),
                version: Some("1.1.0".to_string()),
            }),
        }
    }
}

/// Client capabilities specific to regular expressions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegularExpressionsClientCapabilities {
    /// The engine's name.
    pub engine: String,

    /// The engine's version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Client capabilities specific to the client's markdown parser.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkdownClientCapabilities {
    /// The name of the parser.
    pub parser: String,

    /// The version of the parser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Parameters for the initialize request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeParams {
    /// The process id of the parent process that started the server.
    pub process_id: Option<u32>,

    /// Information about the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_info: Option<ClientInfo>,

    /// The locale the client is currently showing the user interface in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// The rootPath of the workspace. Is null if no folder is open.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,

    /// The rootUri of the workspace. Is null if no folder is open.
    pub root_uri: Option<DocumentUri>,

    /// User provided initialization options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialization_options: Option<serde_json::Value>,

    /// The capabilities provided by the client (editor or tool).
    pub capabilities: ClientCapabilities,

    /// The initial trace setting. If omitted trace is disabled ('off').
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<TraceValue>,

    /// The workspace folders configured in the client when the server starts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
}

/// Information about the client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    /// The name of the client as defined by the client.
    pub name: String,

    /// The client's version as defined by the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// The trace setting for the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceValue {
    /// Turn off tracing.
    Off,
    /// Trace messages only.
    Messages,
    /// Verbose message tracing.
    Verbose,
}

/// A workspace folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    /// The associated URI for this workspace folder.
    pub uri: DocumentUri,

    /// The name of the workspace folder.
    pub name: String,
}

/// The initialize result returned from the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeResult {
    /// The capabilities the language server provides.
    pub capabilities: ServerCapabilities,

    /// Information about the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

/// Information about the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    /// The name of the server as defined by the server.
    pub name: String,

    /// The server's version as defined by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// The capabilities the language server provides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Defines how text documents are synced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<TextDocumentSyncCapability>,

    /// The server provides completion support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_provider: Option<CompletionOptions>,

    /// The server provides hover support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_provider: Option<HoverProviderCapability>,
}

/// Text document sync capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextDocumentSyncCapability {
    Kind(TextDocumentSyncKind),
    Options(TextDocumentSyncOptions),
}

/// Text document sync kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TextDocumentSyncKind {
    /// Documents should not be synced at all.
    None = 0,
    /// Documents are synced by always sending the full content of the document.
    Full = 1,
    /// Documents are synced by sending the full content on open.
    Incremental = 2,
}

/// Text document sync options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentSyncOptions {
    /// Open and close notifications are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_close: Option<bool>,

    /// Change notifications are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<TextDocumentSyncKind>,

    /// If present will save notifications are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save: Option<bool>,

    /// If present will save wait until requests are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save_wait_until: Option<bool>,

    /// If present save notifications are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save: Option<SaveOptions>,
}

/// Save options for text document sync.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveOptions {
    /// The client is supposed to include the content on save.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_text: Option<bool>,
}

/// Completion options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionOptions {
    /// The server provides support to resolve additional information for a completion item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_provider: Option<bool>,

    /// Most tools trigger completion request automatically without explicitly
    /// requesting it using a keyboard shortcut (e.g. Ctrl+Space).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_characters: Option<Vec<String>>,

    /// The list of all possible characters that commit a completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_commit_characters: Option<Vec<String>>,
}

/// Hover provider capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HoverProviderCapability {
    Simple(bool),
    Options(HoverOptions),
}

/// Hover options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoverOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_done_progress: Option<bool>,
}
