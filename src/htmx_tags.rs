use tree_sitter::Point;

use crate::position::PositionDefinition;

/// One tag is just comment in your selected language.
///
/// It looks like this:
/// ```rust
/// fn hello() {
///    // hx@counter
///    println!("hello world");
/// }
/// ```
/// Name should be short, but in the same time have some connection
/// with template part.
///
/// In selected templating language you call this tag:
/// ```html
/// <p hx-lsp="abc"> content </p>
/// ```
/// Most templates you write are longer and it can look this:
/// ```html
/// <input
///       type="text"
///       name="search"
///       hx-lsp="abc"
///       hx-post="/search/"
///       hx-trigger="keyup changed delay:250ms"
///       hx-indicator=".htmx-indicator"
///       hx-target="#todo-results"
///       placeholder="Search"
///       class="bg-white h-10 px-5 pr-10 rounded-full text-2xl focus:outline-none"
///     >
/// ```
/// Your cursor is on line with class attribute values. To avoid jumping manually
/// you can just execute "go to implementation" command in your favorite editor
/// and cursor will be redirected to `hx-lsp` attribute.
#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct Tag {
    /// Start position in Tree
    pub start: Point,
    /// End position in Tree
    pub end: Point,
    /// Name of tag
    pub name: String,
    /// File id
    pub file: usize,
}

/// Check if client current position is in tag range.
pub fn in_tag(line: &str, point: Point) -> Option<Tag> {
    let tag = get_tag(line)?;
    if point >= tag.start && point <= tag.end {
        return Some(tag);
    }
    None
}

/// Return tag from line if it exist. Start and end point,
/// still have no information about line and file. Function get_tags solves this.
pub fn get_tag(line: &str) -> Option<Tag> {
    let parts = line.split("hx@");
    let mut first = parts.filter(|data| !data.contains(' '));
    let first = first.next()?;
    let mut parts = first.split(' ');
    let first = parts.next()?;
    let full = format!("hx@{}", &first);
    let start = line.find(&full)?;
    let end = start + 2 + first.len();
    Some(Tag {
        name: first.to_string(),
        start: Point::new(0, start),
        end: Point::new(0, end),
        file: 0,
    })
}

/// Get all tags from hx-lsp attribute.
pub fn get_tags(value: &str, mut start_char: usize, line: usize) -> Option<Vec<Tag>> {
    if value.starts_with(' ') || value.contains("  ") || value.is_empty() {
        return None;
    }
    let mut tags = vec![];
    let parts = value.split(' ');
    for part in parts {
        let start = start_char;
        let end = start + part.len() - 1;
        start_char = end + 2;
        let tag = Tag {
            name: String::from(part),
            start: Point::new(line, start),
            end: Point::new(line, end),
            file: 0,
        };
        tags.push(tag);
    }
    Some(tags)
}

/// Checks if definition request position is between one of tags.
pub fn in_tags(value: &str, definition: PositionDefinition) -> Option<Tag> {
    let tags = get_tags(value, definition.start, definition.point.row)?;
    for tag in tags {
        let t =
            definition.point.column >= tag.start.row && definition.point.column <= tag.end.column;
        if t {
            return Some(tag);
        }
    }

    None
}
