use log::error;
use lsp_types::TextDocumentPositionParams;
use tree_sitter::{Node, Parser, Point};

use crate::text_store::get_text_document;

#[derive(Debug, Clone, PartialEq)]
enum Position {
    AttributeName(String),
    AttributeValue { name: String, value: String },
}

fn get_text(node: Node<'_>, source: &str) -> String {
    return node
        .utf8_text(source.as_bytes())
        .expect("getting text should never fail")
        .to_string();
}

fn get_attribute_name_and_value(node: Node<'_>, source: &str) -> Option<Position> {
    let value = get_text(node, source);
    let name = get_text(node.prev_named_sibling()?, source);

    return Some(Position::AttributeValue { name, value });
}

fn get_position(root: Node<'_>, source: &str, row: usize, column: usize) -> Option<Position> {
    let desc = root.descendant_for_point_range(
        Point { row, column },
        Point {
            row,
            column: column + 1,
        },
    )?;

    println!("kind {:?}", desc.kind());
    match desc.kind() {
        "attribute" => {
            error!("attribute({}, {}): {:?}", row, column, desc);
            return None;
        }

        "attribute_name" => {
            return Some(Position::AttributeName(get_text(desc, source)));
        }

        "quoted_attribute_value" => {
            return get_attribute_name_and_value(desc, source);
        }
        "attribute_value" => {
            if let Some(parent) = desc.parent() {
                if parent.kind() == "quoted_attribute_value" {
                    return get_attribute_name_and_value(parent, source);
                } else {
                    return get_attribute_name_and_value(desc, source);
                }
            } else {
                error!("why is there no parent??");
            }
        }
        _ => {}
    };

    return None;
}

pub fn get_position_from_lsp_completion(
    text_params: TextDocumentPositionParams,
) -> Option<Position> {
    let text = get_text_document(text_params.text_document.uri)?;
    let pos = text_params.position;

    // TODO: Gallons of perf work can be done starting here
    let mut parser = Parser::new();

    parser
        .set_language(tree_sitter_html::language())
        .expect("could not load html grammer");

    let tree = parser.parse(&text, None).unwrap();
    let root_node = tree.root_node();

    return get_position(
        root_node,
        text.as_str(),
        pos.line as usize,
        pos.character as usize,
    );
}
