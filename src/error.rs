//! Error types for Cirru EDN parsing and manipulation.
//!
//! This module provides detailed error types with position information
//! to help diagnose issues during parsing and deserialization.

use std::fmt;

use cirru_parser::Cirru;

/// Position information in the source text.
///
/// Represents a location in the source code where an error occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
  /// Line number (1-indexed)
  pub line: usize,
  /// Column number (1-indexed)
  pub column: usize,
  /// Byte offset in the source
  pub offset: usize,
}

impl Position {
  /// Create a new position
  pub fn new(line: usize, column: usize, offset: usize) -> Self {
    Position { line, column, offset }
  }

  /// Create a position from byte offset only
  pub fn at_offset(offset: usize) -> Self {
    Position {
      line: 0,
      column: 0,
      offset,
    }
  }
}

impl fmt::Display for Position {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.line > 0 {
      write!(
        f,
        "at line {}, column {} (byte {})",
        self.line, self.column, self.offset
      )
    } else {
      write!(f, "at byte {}", self.offset)
    }
  }
}

/// Errors that can occur during EDN parsing and manipulation.
#[derive(Debug, Clone, PartialEq)]
pub enum EdnError {
  /// Error from the Cirru parser itself (contains original error with full details)
  ParseError {
    /// Original cirru_parser error (preserved for full details)
    original: String,
  },
  /// Invalid EDN structure during data recognition
  StructureError {
    message: String,
    /// Path in the nested Cirru structure (e.g., [0, 2, 1] means root[0][2][1])
    path: Vec<usize>,
    /// Preview of the problematic node in one-liner format
    node_preview: Option<String>,
  },
  /// Invalid value for the expected type
  ValueError {
    message: String,
    /// Path in the nested Cirru structure
    path: Vec<usize>,
    /// Preview of the problematic node
    node_preview: Option<String>,
  },
  /// Deserialization error
  DeserializationError {
    message: String,
    position: Option<Vec<u8>>, // byte position in serialized data
  },
}

impl EdnError {
  /// Create a parse error from cirru_parser error with detailed formatting
  pub fn from_parse_error_detailed(err: cirru_parser::CirruError, source: &str) -> Self {
    EdnError::ParseError {
      original: err.format_detailed(Some(source)),
    }
  }

  /// Create a parse error from cirru_parser error
  pub fn from_parse_error(err: cirru_parser::CirruError) -> Self {
    // Without source code, use format_detailed with None
    EdnError::ParseError {
      original: err.format_detailed(None),
    }
  }

  /// Create a structure error with path and node preview
  pub fn structure(message: impl Into<String>, path: Vec<usize>, node: Option<&Cirru>) -> Self {
    let node_preview = node.and_then(|n| {
      // Use one-liner format for cleaner, more readable output
      cirru_parser::format_expr_one_liner(n).ok()
    });
    EdnError::StructureError {
      message: message.into(),
      path,
      node_preview,
    }
  }

  /// Create a value error with path and node preview
  pub fn value(message: impl Into<String>, path: Vec<usize>, node: Option<&Cirru>) -> Self {
    let node_preview = node.and_then(|n| {
      // Use one-liner format for cleaner, more readable output
      cirru_parser::format_expr_one_liner(n).ok()
    });
    EdnError::ValueError {
      message: message.into(),
      path,
      node_preview,
    }
  }

  /// Create a deserialization error with position
  pub fn deserialization(message: impl Into<String>, position: Option<Vec<u8>>) -> Self {
    EdnError::DeserializationError {
      message: message.into(),
      position,
    }
  }

  /// Get the error message
  pub fn message(&self) -> &str {
    match self {
      EdnError::ParseError { original } => original,
      EdnError::StructureError { message, .. } => message,
      EdnError::ValueError { message, .. } => message,
      EdnError::DeserializationError { message, .. } => message,
    }
  }
}

impl fmt::Display for EdnError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      EdnError::ParseError { original } => {
        write!(f, "Parse error:\n{original}")
      }
      EdnError::StructureError {
        message,
        path,
        node_preview,
      } => {
        write!(f, "Structure error")?;
        if !path.is_empty() {
          write!(f, " at {path:?}")?;
        }
        write!(f, ": {message}")?;
        if let Some(preview) = node_preview {
          write!(f, "\n  Node: {preview}")?;
        }
        Ok(())
      }
      EdnError::ValueError {
        message,
        path,
        node_preview,
      } => {
        write!(f, "Value error")?;
        if !path.is_empty() {
          write!(f, " at {path:?}")?;
        }
        write!(f, ": {message}")?;
        if let Some(preview) = node_preview {
          write!(f, "\n  Node: {preview}")?;
        }
        Ok(())
      }
      EdnError::DeserializationError { message, .. } => {
        write!(f, "Deserialization error: {message}")
      }
    }
  }
}

impl From<&str> for EdnError {
  fn from(message: &str) -> Self {
    EdnError::ParseError {
      original: message.to_string(),
    }
  }
}

// Convert from cirru_parser errors
impl From<cirru_parser::CirruError> for EdnError {
  fn from(err: cirru_parser::CirruError) -> Self {
    EdnError::from_parse_error(err)
  }
}

/// Result type for EDN operations
pub type EdnResult<T> = Result<T, EdnError>;
