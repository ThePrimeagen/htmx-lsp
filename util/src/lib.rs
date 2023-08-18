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

pub fn get_text_byte_offset(source: &str, line: usize, character: usize) -> Option<usize> {

    let mut character = character;
    let mut line = line;
    let mut byte_offset = 0;

    for l in source.lines() {
        if line == 0 {
            byte_offset += character;
            character = character.saturating_sub(l.len());
            break;
        } else {
            line -= 1;
            byte_offset += l.len() + 1;
        }
    }

    if line != 0 || character != 0 {
        return None;
    }

    return Some(byte_offset);
}

