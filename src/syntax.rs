//! Comment syntax per file format.
//!
//! fragments is a format-agnostic primitive: marker pairs are ordinary
//! comments in the *target file's own format*. This module maps a file
//! extension to the comment delimiters used to build and find those
//! markers, so the same fragment can sync into HTML (`<!-- -->`),
//! CSS/JS (`/* */`), YAML/shell (`# …`), SQL/Lua (`-- …`), and more —
//! each target staying valid in its native format.

use std::collections::HashMap;
use std::path::Path;

/// Comment syntax for a file format.
///
/// `close` is empty for line-comment formats (`#`, `--`, `//`): the marker
/// is terminated by end-of-line rather than a closing delimiter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentSyntax {
    pub open: String,
    pub close: String,
}

impl CommentSyntax {
    /// A block comment with an opening and closing delimiter, e.g. `<!--`/`-->`.
    pub fn block(open: &str, close: &str) -> Self {
        Self {
            open: open.to_string(),
            close: close.to_string(),
        }
    }

    /// A line comment with only an opening delimiter, e.g. `#`. The marker
    /// runs to end-of-line.
    pub fn line(open: &str) -> Self {
        Self {
            open: open.to_string(),
            close: String::new(),
        }
    }

    /// True if this is a line comment (no closing delimiter).
    pub fn is_line(&self) -> bool {
        self.close.is_empty()
    }

    /// The opening marker tag, e.g. `<!-- fragment:header -->` or
    /// `# fragment:header`.
    pub fn open_marker(&self, prefix: &str, name: &str) -> String {
        if self.is_line() {
            format!("{} {prefix}:{name}", self.open)
        } else {
            format!("{} {prefix}:{name} {}", self.open, self.close)
        }
    }

    /// The closing marker tag, e.g. `<!-- /fragment:header -->` or
    /// `# /fragment:header`.
    pub fn close_marker(&self, prefix: &str, name: &str) -> String {
        if self.is_line() {
            format!("{} /{prefix}:{name}", self.open)
        } else {
            format!("{} /{prefix}:{name} {}", self.open, self.close)
        }
    }

    /// The search prefix for an opening marker, up to (not including) the
    /// fragment name: `<!-- fragment:` / `# fragment:`.
    pub(crate) fn open_search(&self, prefix: &str) -> String {
        format!("{} {prefix}:", self.open)
    }

    /// The search prefix for a closing marker: `<!-- /fragment:` / `# /fragment:`.
    pub(crate) fn close_search(&self, prefix: &str) -> String {
        format!("{} /{prefix}:", self.open)
    }
}

/// Built-in extension → comment syntax table. The key is a lowercase file
/// extension (no dot), or — for extensionless files like `Makefile` and
/// `Dockerfile` — the lowercase file name.
///
/// Markdown maps to HTML comments because that's what renders invisibly in
/// Markdown. Consumers can extend or override this table via the `[syntax]`
/// section of `fragments.toml` (see [`crate::config::Config`]).
pub fn builtin_syntax(key: &str) -> Option<CommentSyntax> {
    let s = match key {
        // SGML-family: HTML comments. Markdown renders HTML comments invisibly.
        "html" | "htm" | "xhtml" | "xml" | "svg" | "vue" | "svelte" | "md" | "markdown" => {
            CommentSyntax::block("<!--", "-->")
        }
        // C-family block comments.
        "css" | "scss" | "less" | "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "c" | "cc"
        | "cpp" | "cxx" | "h" | "hpp" | "java" | "go" | "rs" | "swift" | "kt" | "kts" | "php"
        | "scala" | "dart" | "rust" => CommentSyntax::block("/*", "*/"),
        // Hash line comments.
        "yaml" | "yml" | "toml" | "sh" | "bash" | "zsh" | "fish" | "py" | "rb" | "pl" | "r"
        | "conf" | "cfg" | "ini" | "properties" | "env" | "dockerfile" | "makefile" | "mk"
        | "gitignore" => CommentSyntax::line("#"),
        // Double-dash line comments.
        "sql" | "lua" | "hs" | "elm" | "adb" | "ads" => CommentSyntax::line("--"),
        _ => return None,
    };
    Some(s)
}

/// The lookup key for a path: its lowercase extension, or — when there is no
/// extension — its lowercase file name (so `Makefile`, `Dockerfile` resolve).
pub fn syntax_key(path: &Path) -> Option<String> {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return Some(ext.to_ascii_lowercase());
    }
    path.file_name()
        .and_then(|f| f.to_str())
        .map(|f| f.to_ascii_lowercase())
}

/// Resolve the comment syntax for a path, consulting `overrides` (from
/// `fragments.toml`) before the built-in table. Returns `None` for formats
/// with no known comment syntax — those files are invisible to fragments.
pub fn resolve(
    path: &Path,
    overrides: &HashMap<String, (String, String)>,
) -> Option<CommentSyntax> {
    let key = syntax_key(path)?;
    if let Some((open, close)) = overrides.get(&key) {
        return Some(CommentSyntax {
            open: open.clone(),
            close: close.clone(),
        });
    }
    builtin_syntax(&key)
}
