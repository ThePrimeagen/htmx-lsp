use ropey::Rope;
use tower_lsp::lsp_types::{Position, Range};
use tree_sitter::{InputEdit, Point};

use crate::htmx_tags::Tag;

pub trait ToInputEdit {
    /// Convert position to point range
    fn to_point(&self) -> (Point, Point);
    /// Based on position in Rope return byte offset.
    fn to_byte(&self, rope: &Rope) -> (usize, usize);
    /// With rope information, get InputEdit changes. This struct is responsible
    /// for incremental updates to TreeSitter.
    fn to_input_edit(&self, rope: &Rope) -> InputEdit;
}

impl ToInputEdit for Range {
    fn to_point(&self) -> (Point, Point) {
        let start = Point::new(self.start.line as usize, self.start.character as usize);
        let end = Point::new(self.end.line as usize, self.end.character as usize);
        (start, end)
    }

    fn to_byte(&self, rope: &Rope) -> (usize, usize) {
        let start_line = rope.line_to_byte(self.start.line as usize);
        let start_offset = start_line + self.start.character as usize;

        let end_line = rope.line_to_byte(self.end.line as usize);
        let end_offset = end_line + self.end.character as usize;

        (start_offset, end_offset)
    }

    fn to_input_edit(&self, rope: &Rope) -> InputEdit {
        let (start_position, new_end_position) = self.to_point();
        let (start_byte, new_end_byte) = self.to_byte(rope);
        InputEdit {
            start_byte,
            old_end_byte: start_byte,
            new_end_byte,
            start_position,
            old_end_position: start_position,
            new_end_position,
        }
    }
}

/// Convert Tag to Positon range for lsp_types.
pub fn to_position(tag: &Tag) -> (Position, Position) {
    (
        Position::new(tag.start.row as u32, tag.start.column as u32),
        Position::new(tag.end.row as u32, tag.end.column as u32),
    )
}
