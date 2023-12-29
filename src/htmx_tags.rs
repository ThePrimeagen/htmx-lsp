use tree_sitter::Point;

use crate::position::PositionDefinition;

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct Tag {
    pub start: Point,
    pub end: Point,
    pub name: String,
    pub file: usize,
}

pub fn in_tag(line: &str, point: Point) -> Option<Tag> {
    let tag = get_tag(line)?;
    if point >= tag.start && point <= tag.end {
        return Some(tag);
    }
    None
}

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

pub fn in_tags(value: &str, definition: PositionDefinition) -> Option<Tag> {
    let tags = get_tags(value, definition.start, definition.line)?;
    for tag in tags {
        let t =
            definition.point.column >= tag.start.row && definition.point.column <= tag.end.column;
        if t {
            return Some(tag);
        }
    }

    None
}
