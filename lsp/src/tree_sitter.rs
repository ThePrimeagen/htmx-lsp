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
    error!("create_attribute tree relation: {:?}", node.to_sexp());

    match node.kind() {
        "\"" => return create_attribute(node.parent()?, source),
        "=" => return create_attribute(node.parent()?, source),
        "attribute" => {
            return create_attribute(node.child(0)?, source);
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

        "fragment" | "element" => {
            let mut cursor = node.walk();
            for child in node.clone().children(&mut cursor) {
                if let Some(result) = create_attribute(child, source) {
                    return Some(result);
                }
            }

            return create_attribute(node.child(0)?, source);
        }

        // Example of an ERROR node
        // (ERROR (tag_name) (attribute_name))
        "ERROR" => {
            let mut cursor = node.walk();
            for child in node.clone().children(&mut cursor) {
                if let Some(attribute) = create_attribute(child, source) {
                    return Some(attribute);
                }
            }
        }

        // Example of an self_closing_tag node
        // (self_closing_tag (tag_name) (attribute (attribute_name)) (MISSING \"/>\"))
        // (start_tag (tag_name) (attribute (attribute_name)) (MISSING \">\"))
        "self_closing_tag" | "start_tag" => {
            let mut cursor = node.walk();
            for child in node.clone().children(&mut cursor) {
                if child.kind() == "attribute" {
                    return create_attribute(child.child(0 as usize)?, source);
                }
            }
        }
        _ => {}
    };
    return None;
}

fn get_position(root: Node<'_>, source: &str, row: usize, column: usize) -> Option<Position> {
    let desc = root.descendant_for_point_range(Point { row, column }, Point { row, column })?;

    return create_attribute(desc, source);
}

pub fn get_position_from_lsp_completion(
    text_params: TextDocumentPositionParams,
) -> Option<Position> {
    error!("get_position_from_lsp_completion");
    let text = get_text_document(text_params.text_document.uri)?;
    error!("get_position_from_lsp_completion: text {}", text);
    let pos = text_params.position;
    error!("get_position_from_lsp_completion: pos {:?}", pos);

    // TODO: Gallons of perf work can be done starting here
    let mut parser = Parser::new();

    parser
        .set_language(tree_sitter_html::language())
        .expect("could not load html grammer");

    let tree = parser.parse(&text, None)?;
    let root_node = tree.root_node();

    return get_position(
        root_node,
        text.as_str(),
        pos.line as usize,
        pos.character as usize,
    );
}
