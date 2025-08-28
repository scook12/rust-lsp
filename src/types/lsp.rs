//! Core LSP types as defined by the Language Server Protocol specification.
//!
//! This module contains the basic data structures used throughout the LSP,
//! such as positions, ranges, locations, and other fundamental types.

use crate::types::{DocumentUri, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Position in a text document expressed as zero-based line and character offset.
/// The offsets are based on a UTF-16 string representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Line position in a document (zero-based).
    pub line: u32,
    /// Character offset on a line in a document (zero-based).
    /// Assuming that the line is represented as a string, the `character` value
    /// represents the gap between the `character` and `character + 1`.
    pub character: u32,
}

impl Position {
    /// Create a new position.
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }

    /// Create a position at the start of a document.
    pub fn start() -> Self {
        Self::new(0, 0)
    }
}

/// A range in a text document expressed as (zero-based) start and end positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    /// The range's start position.
    pub start: Position,
    /// The range's end position.
    pub end: Position,
}

impl Range {
    /// Create a new range.
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a range from line/character coordinates.
    pub fn from_coords(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Self {
        Self::new(
            Position::new(start_line, start_char),
            Position::new(end_line, end_char),
        )
    }

    /// Create a single-character range at the given position.
    pub fn single_char(position: Position) -> Self {
        Self::new(
            position,
            Position::new(position.line, position.character + 1),
        )
    }

    /// Check if this range contains the given position.
    pub fn contains(&self, position: Position) -> bool {
        position >= self.start && position < self.end
    }

    /// Check if this range is empty (start equals end).
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Location {
    /// The resource's URI.
    pub uri: DocumentUri,
    /// The range in the document.
    pub range: Range,
}

impl Location {
    /// Create a new location.
    pub fn new(uri: impl Into<DocumentUri>, range: Range) -> Self {
        Self {
            uri: uri.into(),
            range,
        }
    }
}

/// Represents a link between a source and a target location.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationLink {
    /// Span of the origin of this link.
    /// Used as the underlined span for mouse interaction. Defaults to the word range at
    /// the mouse position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_selection_range: Option<Range>,

    /// The target resource identifier of this link.
    pub target_uri: DocumentUri,

    /// The full target range of this link. If the target for example is a symbol then
    /// target range is the range enclosing this symbol not including leading/trailing
    /// whitespace but everything else like comments.
    pub target_range: Range,

    /// The range that should be selected and revealed when this link is being followed,
    /// e.g. the name of a function. Must be contained by the `target_range`.
    pub target_selection_range: Range,
}

/// Defines a diagnostic, such as a compiler error or warning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// The range at which the message applies.
    pub range: Range,

    /// The diagnostic's severity. Can be omitted. If omitted it is up to the
    /// client to interpret diagnostics as error, warning, info or hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<DiagnosticSeverity>,

    /// The diagnostic's code, which usually appear in the user interface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<DiagnosticCode>,

    /// An optional property to describe the error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_description: Option<CodeDescription>,

    /// A human-readable string describing the source of this diagnostic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// The diagnostic's message.
    pub message: String,

    /// Additional metadata about the diagnostic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<DiagnosticTag>>,

    /// An array of related diagnostic information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,

    /// A data entry field that is preserved between a textDocument/publishDiagnostics
    /// notification and textDocument/codeAction request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// The diagnostic's severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    /// Reports an error.
    Error = 1,
    /// Reports a warning.
    Warning = 2,
    /// Reports an information.
    Information = 3,
    /// Reports a hint.
    Hint = 4,
}

impl Serialize for DiagnosticSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for DiagnosticSeverity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            1 => Ok(DiagnosticSeverity::Error),
            2 => Ok(DiagnosticSeverity::Warning),
            3 => Ok(DiagnosticSeverity::Information),
            4 => Ok(DiagnosticSeverity::Hint),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid diagnostic severity: {}",
                value
            ))),
        }
    }
}

/// The diagnostic's code.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticCode {
    Number(i32),
    String(String),
}

/// Structure to capture a description for an error code.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CodeDescription {
    /// A URI to open with more information about the diagnostic error.
    pub href: Uri,
}

/// The diagnostic tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DiagnosticTag {
    /// Unused or unnecessary code.
    Unnecessary = 1,
    /// Deprecated or obsolete code.
    Deprecated = 2,
}

impl Serialize for DiagnosticTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for DiagnosticTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            1 => Ok(DiagnosticTag::Unnecessary),
            2 => Ok(DiagnosticTag::Deprecated),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid diagnostic tag: {}",
                value
            ))),
        }
    }
}

/// Represents a related message and source code location for a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    /// The location of this related diagnostic information.
    pub location: Location,
    /// The message of this related diagnostic information.
    pub message: String,
}

/// A command is returned from the server to the client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
    /// Title of the command, like 'save'.
    pub title: String,
    /// The identifier of the actual command handler.
    pub command: String,
    /// Arguments that the command handler should be invoked with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<serde_json::Value>>,
}

/// A text edit applicable to a text document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextEdit {
    /// The range of the text document to be manipulated.
    pub range: Range,
    /// The string to be inserted. For delete operations use an empty string.
    pub new_text: String,
}

impl TextEdit {
    /// Create a new text edit.
    pub fn new(range: Range, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }

    /// Create a text edit that inserts text at a position.
    pub fn insert(position: Position, text: impl Into<String>) -> Self {
        Self::new(Range::new(position, position), text)
    }

    /// Create a text edit that deletes a range.
    pub fn delete(range: Range) -> Self {
        Self::new(range, "")
    }

    /// Create a text edit that replaces a range with new text.
    pub fn replace(range: Range, new_text: impl Into<String>) -> Self {
        Self::new(range, new_text)
    }
}

/// Additional information that describes document changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangeAnnotation {
    /// A human-readable string describing the actual change.
    pub label: String,

    /// A flag which indicates that user confirmation is needed before applying the change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs_confirmation: Option<bool>,

    /// A human-readable string which is rendered less prominent in the user interface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// An identifier referring to a change annotation managed by a workspace edit.
pub type ChangeAnnotationIdentifier = String;

/// A special text edit with an additional change annotation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnnotatedTextEdit {
    /// The range of the text document to be manipulated.
    pub range: Range,
    /// The string to be inserted.
    pub new_text: String,
    /// The actual identifier of the change annotation.
    pub annotation_id: ChangeAnnotationIdentifier,
}

/// Describes textual changes on a text document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextDocumentEdit {
    /// The text document to change.
    pub text_document: OptionalVersionedTextDocumentIdentifier,
    /// The edits to be applied.
    pub edits: Vec<OneOf<TextEdit, AnnotatedTextEdit>>,
}

/// A generic resource operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ResourceOperation {
    #[serde(rename = "create")]
    Create(CreateFile),
    #[serde(rename = "rename")]
    Rename(RenameFile),
    #[serde(rename = "delete")]
    Delete(DeleteFile),
}

/// Create file operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CreateFile {
    /// A create file operation.
    pub kind: String, // "create"
    /// The resource to create.
    pub uri: DocumentUri,
    /// Additional options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<CreateFileOptions>,
    /// An optional annotation identifier describing the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_id: Option<ChangeAnnotationIdentifier>,
}

/// Options to create a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CreateFileOptions {
    /// Overwrite existing file. Overwrite wins over `ignore_if_exists`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
    /// Ignore if exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_exists: Option<bool>,
}

/// Rename file operation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenameFile {
    /// A rename file operation
    pub kind: String, // "rename"
    /// The old (existing) location.
    pub old_uri: DocumentUri,
    /// The new location.
    pub new_uri: DocumentUri,
    /// Rename options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<RenameFileOptions>,
    /// An optional annotation identifier describing the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_id: Option<ChangeAnnotationIdentifier>,
}

/// Rename file options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenameFileOptions {
    /// Overwrite target if existing. Overwrite wins over `ignore_if_exists`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
    /// Ignores if target exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_exists: Option<bool>,
}

/// Delete file operation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeleteFile {
    /// A delete file operation
    pub kind: String, // "delete"
    /// The file to delete.
    pub uri: DocumentUri,
    /// Delete options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<DeleteFileOptions>,
    /// An optional annotation identifier describing the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_id: Option<ChangeAnnotationIdentifier>,
}

/// Delete file options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeleteFileOptions {
    /// Delete the content recursively if a folder is denoted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,
    /// Ignore the operation if the file doesn't exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_not_exists: Option<bool>,
}

/// A workspace edit represents changes to many resources managed in the workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    /// Holds changes to existing resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<HashMap<DocumentUri, Vec<TextEdit>>>,

    /// Depending on the client capability `workspace.workspaceEdit.resourceOperations` document changes
    /// are either an array of `TextDocumentEdit`s to express changes to n different text documents
    /// where each text document edit addresses a specific version of a text document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_changes: Option<Vec<DocumentChange>>,

    /// A map of change annotations that can be referenced in `AnnotatedTextEdit`s or create, rename and
    /// delete file / folder operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_annotations: Option<HashMap<ChangeAnnotationIdentifier, ChangeAnnotation>>,
}

/// Document change type for workspace edits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DocumentChange {
    TextDocumentEdit(TextDocumentEdit),
    ResourceOperation(ResourceOperation),
}

/// Text documents are identified using a URI.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    /// The text document's URI.
    pub uri: DocumentUri,
}

impl TextDocumentIdentifier {
    /// Create a new text document identifier.
    pub fn new(uri: impl Into<DocumentUri>) -> Self {
        Self { uri: uri.into() }
    }
}

/// A versioned text document identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    /// The text document's URI.
    pub uri: DocumentUri,
    /// The version number of this document.
    pub version: i32,
}

impl VersionedTextDocumentIdentifier {
    /// Create a new versioned text document identifier.
    pub fn new(uri: impl Into<DocumentUri>, version: i32) -> Self {
        Self {
            uri: uri.into(),
            version,
        }
    }
}

/// A text document identifier where the version is optional.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OptionalVersionedTextDocumentIdentifier {
    /// The text document's URI.
    pub uri: DocumentUri,
    /// The version number of this document. If a versioned text document identifier
    /// is sent from the server to the client and the file is not open in the editor
    /// (the server has not received an open notification before) the server can send
    /// `null` to indicate that the version is unknown and the content on disk is the
    /// truth (as specified with document content ownership).
    pub version: Option<i32>,
}

impl OptionalVersionedTextDocumentIdentifier {
    /// Create a new optional versioned text document identifier.
    pub fn new(uri: impl Into<DocumentUri>, version: Option<i32>) -> Self {
        Self {
            uri: uri.into(),
            version,
        }
    }
}

/// A helper type to represent either one of two types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOf<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> OneOf<A, B> {
    pub fn is_left(&self) -> bool {
        matches!(self, OneOf::Left(_))
    }

    pub fn is_right(&self) -> bool {
        matches!(self, OneOf::Right(_))
    }

    pub fn left(self) -> Option<A> {
        match self {
            OneOf::Left(a) => Some(a),
            OneOf::Right(_) => None,
        }
    }

    pub fn right(self) -> Option<B> {
        match self {
            OneOf::Right(b) => Some(b),
            OneOf::Left(_) => None,
        }
    }
}

impl<A> From<A> for OneOf<A, ()> {
    fn from(a: A) -> Self {
        OneOf::Left(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_ordering() {
        let pos1 = Position::new(1, 5);
        let pos2 = Position::new(1, 10);
        let pos3 = Position::new(2, 0);

        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
        assert!(pos1 < pos3);
    }

    #[test]
    fn test_range_contains() {
        let range = Range::new(Position::new(1, 5), Position::new(1, 10));

        assert!(range.contains(Position::new(1, 7)));
        assert!(!range.contains(Position::new(1, 4)));
        assert!(!range.contains(Position::new(1, 10))); // end is exclusive
        assert!(!range.contains(Position::new(2, 0)));
    }

    #[test]
    fn test_range_is_empty() {
        let pos = Position::new(1, 5);
        let empty_range = Range::new(pos, pos);
        let non_empty_range = Range::new(pos, Position::new(1, 6));

        assert!(empty_range.is_empty());
        assert!(!non_empty_range.is_empty());
    }

    #[test]
    fn test_text_edit_operations() {
        let pos = Position::new(1, 5);
        let range = Range::new(pos, Position::new(1, 10));

        let insert = TextEdit::insert(pos, "text");
        assert_eq!(insert.range.start, insert.range.end);

        let delete = TextEdit::delete(range);
        assert_eq!(delete.new_text, "");

        let replace = TextEdit::replace(range, "new");
        assert_eq!(replace.new_text, "new");
        assert_eq!(replace.range, range);
    }

    #[test]
    fn test_diagnostic_severity_values() {
        // Test that the enum values match the LSP specification
        assert_eq!(DiagnosticSeverity::Error as u8, 1);
        assert_eq!(DiagnosticSeverity::Warning as u8, 2);
        assert_eq!(DiagnosticSeverity::Information as u8, 3);
        assert_eq!(DiagnosticSeverity::Hint as u8, 4);
    }

    #[test]
    fn test_one_of_type() {
        let left: OneOf<i32, String> = OneOf::Left(42);
        let right: OneOf<i32, String> = OneOf::Right("test".to_string());

        assert!(left.is_left());
        assert!(!left.is_right());
        assert!(!right.is_left());
        assert!(right.is_right());

        assert_eq!(left.left(), Some(42));
        assert_eq!(right.right(), Some("test".to_string()));
    }
}
