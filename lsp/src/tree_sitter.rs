use log::error;
use lsp_types::TextDocumentPositionParams;
use tree_sitter::{Node, Parser, Point};

use crate::text_store::get_text_document;

#[derive(Debug, Clone, PartialEq)]
pub enum Position {
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

fn create_attribute(node: Node<'_>, source: &str) -> Option<Position> {
    match node.kind() {
        "\"" => return create_attribute(node.parent()?, source),
        "=" => return create_attribute(node.parent()?, source),
        "attribute" => {
            return None;
        }

        "attribute_name" => {
            return Some(Position::AttributeName(get_text(node, source)));
        }

        "quoted_attribute_value" => {
            return get_attribute_name_and_value(node, source);
        }
        "attribute_value" => {
            if let Some(parent) = node.parent() {
                if parent.kind() == "quoted_attribute_value" {
                    return get_attribute_name_and_value(parent, source);
                } else {
                    return get_attribute_name_and_value(node, source);
                }
            } else {
                error!("why is there no parent??");
                todo!("should fix this issue");
            }
        }
        _ => {}
    };
    return None
}

fn get_position(root: Node<'_>, source: &str, row: usize, column: usize) -> Option<Position> {
    let desc = root.descendant_for_point_range(
        Point { row, column },
        Point {
            row,
            column: column + 1,
        },
    )?;

    return create_attribute(desc, source)
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
        pos.character as usize - 1,
    );
}
