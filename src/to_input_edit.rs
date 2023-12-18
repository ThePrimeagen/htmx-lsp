use ropey::Rope;
use tower_lsp::lsp_types::Range;
use tree_sitter::{InputEdit, Point};

pub trait ToInputEdit {
    fn to_point(&self) -> (Point, Point);
    fn to_byte(&self, rope: &Rope) -> (usize, usize);
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
