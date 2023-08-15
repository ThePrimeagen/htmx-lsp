use lsp_types::Position;

pub fn byte_pos_to_line_col(source: &str, byte_pos: usize) -> Position {
    let mut line = 0;
    let mut col: u32 = 0;

    for (idx, c) in source.chars().enumerate() {
        if idx >= byte_pos {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    return Position {
        line,
        character: col.saturating_sub(1),
    };
}

