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

/// Format a nested Cirru path using the conventional `@1.2.3` notation,
/// always absolute from the document root so it can be reliably parsed
/// by tools/agents (never mixed with prose like "position N").
fn format_path(path: &[usize]) -> String {
  let mut result = String::from("@");
  for (index, item) in path.iter().enumerate() {
    if index > 0 {
      result.push('.');
    }
    result.push_str(&item.to_string());
  }
  result
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

  /// Create a structure error with path and node preview, using `focus_path`
  /// for structural folding (relative to `node`) while `path` is used for display.
  pub fn structure_focused(
    message: impl Into<String>,
    path: Vec<usize>,
    focus_path: &[usize],
    node: Option<&Cirru>,
  ) -> Self {
    let node_preview = node.and_then(|n| {
      let folded = cirru_parser::focus_cirru_preview(n, focus_path);
      cirru_parser::format(std::slice::from_ref(&folded), false.into())
        .ok()
        .map(|s| s.trim().to_string())
    });
    EdnError::StructureError {
      message: message.into(),
      path,
      node_preview,
    }
  }

  /// Create a structure error with path and node preview.
  /// The node is structurally folded along `path` before formatting.
  pub fn structure(message: impl Into<String>, path: Vec<usize>, node: Option<&Cirru>) -> Self {
    let node_preview = node.and_then(|n| {
      let folded = cirru_parser::focus_cirru_preview(n, &path);
      cirru_parser::format(std::slice::from_ref(&folded), false.into())
        .ok()
        .map(|s| s.trim().to_string())
    });
    EdnError::StructureError {
      message: message.into(),
      path,
      node_preview,
    }
  }

  /// Create a value error with path and node preview.
  /// The node is structurally folded along `path` before formatting.
  pub fn value(message: impl Into<String>, path: Vec<usize>, node: Option<&Cirru>) -> Self {
    let node_preview = node.and_then(|n| {
      let folded = cirru_parser::focus_cirru_preview(n, &path);
      cirru_parser::format(std::slice::from_ref(&folded), false.into())
        .ok()
        .map(|s| s.trim().to_string())
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

  /// Wrap an inner `EdnError` with a breadcrumb prefix describing the outer
  /// context (e.g. which map key or record field it was found under),
  /// WITHOUT re-formatting the inner error's own "Structure error at ..."
  /// header. This keeps the final message to a single coordinate + a
  /// readable breadcrumb trail, and reuses the innermost (most specific)
  /// node preview instead of stacking multiple partial previews.
  ///
  /// The resulting error's `path` is the inner error's path when available
  /// (already absolute from the document root), falling back to `fallback_path`
  /// otherwise (e.g. when wrapping a `DeserializationError`).
  pub fn wrap_structure(crumb: impl Into<String>, fallback_path: Vec<usize>, inner: &EdnError) -> Self {
    let crumb = crumb.into();
    match inner {
      EdnError::StructureError {
        message,
        path,
        node_preview,
      }
      | EdnError::ValueError {
        message,
        path,
        node_preview,
      } => EdnError::StructureError {
        message: format!("{crumb}: {message}"),
        path: path.clone(),
        node_preview: node_preview.clone(),
      },
      other => EdnError::StructureError {
        message: format!("{crumb}: {}", other.message()),
        path: fallback_path,
        node_preview: None,
      },
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
          write!(f, " at {}", format_path(path))?;
        }
        write!(f, ": ")?;
        for (i, line) in message.lines().enumerate() {
          if i > 0 {
            write!(f, "\n  ")?;
          }
          write!(f, "{line}")?;
        }
        if let Some(preview) = node_preview {
          for line in preview.lines() {
            write!(f, "\n  {line}")?;
          }
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
          write!(f, " at {}", format_path(path))?;
        }
        write!(f, ": ")?;
        for (i, line) in message.lines().enumerate() {
          if i > 0 {
            write!(f, "\n  ")?;
          }
          write!(f, "{line}")?;
        }
        if let Some(preview) = node_preview {
          for line in preview.lines() {
            write!(f, "\n  {line}")?;
          }
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

#[cfg(test)]
mod tests {
  use super::format_path;

  #[test]
  fn formats_paths_with_at_and_dot_separators() {
    assert_eq!(format_path(&[1, 2, 3, 4]), "@1.2.3.4");
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
