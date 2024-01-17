use crate::{
    text_store::DOCUMENT_STORE,
    tree_sitter_querier::{query_attr_keys_for_completion, query_attr_values_for_completion},
};
use log::{debug, error};
use lsp_textdocument::FullTextDocument;
use lsp_types::{TextDocumentContentChangeEvent, TextDocumentPositionParams};
use tree_sitter::{InputEdit, Node, Point};

#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    AttributeName(String),
    AttributeValue { name: String, value: String },
}

// TODO: remove if not used
#[allow(dead_code)]
fn get_text(node: Node<'_>, source: &str) -> String {
    node.utf8_text(source.as_bytes())
        .expect("getting text should never fail")
        .to_string()
}

// TODO: remove if not used
#[allow(dead_code)]
fn get_attribute_name_and_value(node: Node<'_>, source: &str) -> Option<Position> {
    let value = get_text(node, source);
    let name = get_text(node.prev_named_sibling()?, source);

    Some(Position::AttributeValue { name, value })
}

// TODO: remove if not used
#[allow(dead_code)]
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
    None
}

fn find_element_referent_to_current_node(node: Node<'_>) -> Option<Node<'_>> {
    debug!("node {:?}", node.to_sexp());
    if node.kind() == "element" || node.kind() == "fragment" {
        return Some(node);
    }

    find_element_referent_to_current_node(node.parent()?)
}

fn query_position(root: Node<'_>, source: &str, trigger_point: Point) -> Option<Position> {
    debug!("query_position root {:?}", root.to_sexp());
    let closest_node = root.descendant_for_point_range(trigger_point, trigger_point)?;
    debug!("query_position closest_node {:?}", closest_node.to_sexp());

    let element = find_element_referent_to_current_node(closest_node)?;

    let attr_completion = query_attr_keys_for_completion(element, source, trigger_point);

    if attr_completion.is_some() {
        return attr_completion;
    }

    query_attr_values_for_completion(element, source, trigger_point)
}

// TODO: remove if not used
#[allow(dead_code)]
fn get_position(root: Node<'_>, source: &str, row: usize, column: usize) -> Option<Position> {
    error!("get_position");

    let desc = root.descendant_for_point_range(Point { row, column }, Point { row, column })?;

    error!("get_position: desc {:?}", desc);

    create_attribute(desc, source)
}

pub fn get_position_from_lsp_completion(
    text_params: TextDocumentPositionParams,
) -> Option<Position> {
    error!("get_position_from_lsp_completion");
    let pos = text_params.position;
    error!("get_position_from_lsp_completion: pos {:?}", pos);

    if let Some(entry) = DOCUMENT_STORE
        .get()
        .expect("text store not initialized")
        .lock()
        .expect("text store mutex poisoned")
        .get_mut(text_params.text_document.uri.as_str())
    {
        let text = entry.doc.get_content(None);
        entry.tree = entry.parser.parse(text, entry.tree.as_ref());

        if let Some(ref curr_tree) = entry.tree {
            let trigger_point = Point::new(pos.line as usize, pos.character as usize);
            return query_position(curr_tree.root_node(), text, trigger_point);
        }
    }

    None
}

/// Convert an `lsp_types::TextDocumentContentChangeEvent` to a `tree_sitter::InputEdit`
pub fn text_doc_change_to_ts_edit(
    change: &TextDocumentContentChangeEvent,
    doc: &FullTextDocument,
) -> Result<InputEdit, &'static str> {
    let range = change.range.ok_or("Invalid edit range")?;
    let start = range.start;
    let end = range.end;

    let start_byte = doc.offset_at(start) as usize;
    let new_end_byte = start_byte + change.text.len();
    let new_end_pos = doc.position_at(new_end_byte as u32);

    Ok(InputEdit {
        start_byte,
        old_end_byte: doc.offset_at(end) as usize,
        new_end_byte,
        start_position: Point {
            row: start.line as usize,
            column: start.character as usize,
        },
        old_end_position: Point {
            row: end.line as usize,
            column: end.character as usize,
        },
        new_end_position: Point {
            row: new_end_pos.line as usize,
            column: new_end_pos.character as usize,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::{get_position, query_position, Position};
    use tree_sitter::{Parser, Point};

    fn prepare_tree(text: &str) -> tree_sitter::Tree {
        let language = tree_sitter_html::language();
        let mut parser = Parser::new();

        parser
            .set_language(language)
            .expect("could not load html grammer");

        parser.parse(text, None).expect("not to fail")
    }

    #[test]
    fn test_it_suggests_attr_names_when_starting_tag() {
        let text = r##"<div hx- ></div>"##;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 8));
        // Fixes issue with not suggesting hx-* attributes
        // let expected = get_position(tree.root_node(), text, 0, 8);
        // assert_eq!(matches, expected);
        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn test_it_does_not_suggest_when_quote_not_initiated() {
        let text = r##"<div hx-swap= ></div>"##;

        let tree = prepare_tree(text);

        let expected = get_position(tree.root_node(), text, 0, 13);
        let matches = query_position(tree.root_node(), text, Point::new(0, 13));

        assert_eq!(matches, expected);
        assert_eq!(matches, None);
    }

    #[test]
    fn test_it_suggests_attr_values_when_starting_quote_value() {
        let text = r#"<div hx-swap=" ></div>"#;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 14));

        // The new implementation doesn't return incomplete tags as value :)
        // let expected = get_position(tree.root_node(), text, 0, 14);
        // assert_eq!(matches, expected);
        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-swap".to_string(),
                value: "".to_string()
            })
        );
    }

    #[test]
    fn test_it_suggests_attr_values_when_open_and_closed_quotes() {
        let text = r#"<div hx-swap=""></div>"#;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 14));

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-swap".to_string(),
                value: "".to_string()
            })
        );
    }

    #[test]
    fn test_it_suggests_attr_values_once_opening_quotes_in_between_tags() {
        let text = r#"<div id="fa" hx-swap="hx-swap" hx-swap="hx-swap">
      <span hx-target="
      <button>Click me</button>
    </div>
    "#;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(1, 23));

        // The new implementation doesn't return incomplete tags as value :)
        // let expected = get_position(tree.root_node(), text, 1, 16);
        // assert_eq!(matches, expected);
        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "".to_string()
            })
        );
    }

    #[test]
    fn test_it_suggests_attr_names_for_incomplete_attr_in_between_tags() {
        let text = r#"<div id="fa" hx-target="this" hx-swap="hx-swap">
      <span hx-
      <button>Click me</button>
    </div>
    "#;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(1, 14));

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn test_it_matches_more_than_one_attribute() {
        let text = r#"<div hx-get="/foo" hx-target="this" hx- ></div>"#;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 39));

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn test_it_suggests_attr_value_when_attr_is_empty_and_in_between_attributes() {
        let text = r##"<div hx-get="/foo" hx-target="" hx-swap="#swap"></div>
    "##;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 30));

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "".to_string()
            })
        );
    }

    #[test]
    fn test_it_suggests_attr_values_for_incoplete_quoted_attr_when_in_between_attributes() {
        let text = r##"<div hx-get="/foo" hx-target=" hx-swap="#swap"></div>"##;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 30));

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "".to_string()
            })
        );
    }

    #[test]
    fn test_it_suggests_attr_names_for_incoplete_quoted_value_in_between_attributes() {
        let text = r##"<div hx-get="/foo" hx- hx-swap="#swap"></div>
        <span class="foo" />"##;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 22));

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn test_it_suggests_attribute_keys_when_half_completeded() {
        let text = r##"<div hx-get="/foo" hx-t hx-swap="#swap"></div>
        <span class="foo" />"##;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 23));

        assert_eq!(matches, Some(Position::AttributeName("hx-t".to_string())));
    }

    #[test]
    fn test_it_suggests_values_for_already_filled_attributes() {
        let text = r##"<div hx-get="/foo" hx-target="find " hx-swap="#swap"></div>"##;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 35));

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "".to_string()
            })
        );
    }

    #[test]
    fn test_it_does_not_suggest_when_cursor_isnt_within_a_htmx_attribute() {
        let text = r#"<div hx-get="/foo"  class="p-4" ></div>"#;

        let tree = prepare_tree(text);

        let matches = query_position(tree.root_node(), text, Point::new(0, 19));

        assert_eq!(matches, None);
    }
}
