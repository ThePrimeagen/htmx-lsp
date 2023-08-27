use log::{debug, error};
use lsp_types::TextDocumentPositionParams;
use std::collections::HashMap;
use tree_sitter::{Node, Parser, Point, Query, QueryCursor};

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
    return None;
}

fn get_position_by_query(query: &str, node: Node<'_>, source: &str) -> Option<Position> {
    let query = Query::new(tree_sitter_html::language(), query)
        .expect(&format!("get_position_by_query invalid query {query}"));
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, node, source.as_bytes());

    debug!("get_position_by_query node {:?}", node.to_sexp());

    // TODO this only suggestion for matches at the end of the tag!
    let first_match = matches.last()?;

    let capture_names = query.capture_names();
    debug!("get_position_by_query captures {:?}", capture_names);

    debug!("get_position_by_query first_match {:?}", first_match);

    let props = first_match
        .captures
        .iter()
        .fold(HashMap::new(), |mut props, capture| {
            let name = capture_names[capture.index as usize].to_owned();
            let value = capture
                .node
                .utf8_text(source.as_bytes())
                .expect(&format!("failed to parse capture value for '{name}'"))
                .to_owned();

            props.insert(name, value);

            props
        });

    let name = props.get("attr_name")?.to_owned();

    let attr_value = match props.get("attr_value").or(props.get("quoted_attr_value")) {
        Some(value) => Some(value.to_owned()),
        None => {
            return Some(Position::AttributeName(name.to_owned()));
        }
    };

    let value = match props.get("incomplete_tag") {
        Some(_) => "".to_owned(), // otherwise it gets other tags as string
        None => attr_value.expect("at this point value must be present"),
    };

    return Some(Position::AttributeValue { name, value });
}

fn find_element_referent_to_current_node(node: Node<'_>) -> Option<Node<'_>> {
    debug!("node {:?}", node.to_sexp());
    if node.kind() == "element" || node.kind() == "fragment" {
        return Some(node);
    }

    return find_element_referent_to_current_node(node.parent()?);
}

fn query_position(root: Node<'_>, source: &str, row: usize, column: usize) -> Option<Position> {
    debug!("query_position");

    debug!("query_position root {:?}", root.to_sexp());
    let closest_node =
        root.descendant_for_point_range(Point { row, column }, Point { row, column })?;
    debug!("query_position closest_node {:?}", closest_node.to_sexp());

    let node = find_element_referent_to_current_node(closest_node)?;

    // Maybe there is a better way to check for this
    // usually an ERROR node means the there is an incomplete tag at some
    // point in the tree
    if node.to_sexp().contains("ERROR") {
        // See: https://tree-sitter.github.io/tree-sitter/using-parsers#query-syntax
        return get_position_by_query(
            r#"
            (
              (_
                (ERROR 
                    (tag_name)
                    (attribute_name) @attr_name
                    (attribute_value)? @attr_value
                ) @incomplete_tag
              )
                (#match? @attr_name "hx-.*=?")
            )"#,
            node,
            source,
        );
    }

    // See: https://tree-sitter.github.io/tree-sitter/using-parsers#query-syntax
    return get_position_by_query(
        r#"(
          (_
            (_ 
              (tag_name)
              (attribute
                (attribute_name) @attr_name
                (quoted_attribute_value 
                  (attribute_value)? @attr_value
                )? @quoted_attr_value
              )
            ) @tag
          )

          (#match? @attr_name "hx-.*=?")
        )"#,
        node,
        source,
    );
}

fn get_position(root: Node<'_>, source: &str, row: usize, column: usize) -> Option<Position> {
    error!("get_position");

    let desc = root.descendant_for_point_range(Point { row, column }, Point { row, column })?;

    error!("get_position: desc {:?}", desc);

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

    return query_position(
        root_node,
        text.as_str(),
        pos.line as usize,
        pos.character as usize,
    );
}

#[cfg(test)]
mod tests {
    use super::{get_position, query_position, Position};
    use tree_sitter::{Parser, Point, Query, QueryCursor};

    fn prepare_tree(text: &str) -> tree_sitter::Tree {
        let language = tree_sitter_html::language();
        let mut parser = Parser::new();

        parser
            .set_language(language)
            .expect("could not load html grammer");

        let tree = parser.parse(&text, None).expect("not to fail");

        return tree;
    }

    #[test]
    fn test_it_matches_when_starting_tag() {
        let text = r##"<div hx- ></div>"##;

        let tree = prepare_tree(&text);

        let matches = query_position(tree.root_node(), text, 0, 4);
        // Fixes issue with not suggesting hx-* attributes
        // let expected = get_position(tree.root_node(), text, 0, 4);
        // assert_eq!(matches, expected);
        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn test_does_not_match_when_quote_not_initiated() {
        let text = r##"<div hx-swap= ></div>"##;

        let tree = prepare_tree(&text);

        let expected = get_position(tree.root_node(), text, 0, 14);
        let matches = query_position(tree.root_node(), text, 0, 14);

        assert_eq!(matches, expected);
        assert_eq!(matches, None);
    }

    #[test]
    fn test_it_matches_when_starting_quote_value() {
        let text = r##"<div hx-swap=" ></div>"##;

        let tree = prepare_tree(&text);

        let matches = query_position(tree.root_node(), text, 0, 14);

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
    fn test_it_matches_when_open_and_closed_quotes() {
        let text = r##"<div hx-swap=""></div>"##;

        let tree = prepare_tree(&text);

        let expected = get_position(tree.root_node(), text, 0, 14);
        let matches = query_position(tree.root_node(), text, 0, 14);

        assert_eq!(matches, expected);
        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-swap".to_string(),
                value: "\"\"".to_string()
            })
        );
    }

    #[test]
    fn test_it_matches_a_unclosed_tag_in_the_middle() {
        let text = r##"<div id="fa" hx-swap="hx-swap" hx-swap="hx-swap">
      <span hx-target="
      <button>Click me</button>
    </div>
    "##;

        let tree = prepare_tree(&text);

        let matches = query_position(tree.root_node(), text, 1, 16);

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
    fn test_it_matches_a_unclosed_tag_in_the_middle_() {
        let text = r##"<div id="fa" hx-target="this" hx-swap="hx-swap">
      <span hx-
      <tebutton>Click me</button>
    </div>
    "##;

        let tree = prepare_tree(&text);

        let matches = query_position(tree.root_node(), text, 1, 14);

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn test_it_matches_more_than_one_attribute() {
        let text = r##"<div hx-get="/foo" hx-target="this" hx->
    </div>
    "##;

        let tree = prepare_tree(&text);

        let matches = query_position(tree.root_node(), text, 1, 39);

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    // skip - This test when suggesting an attribute in the middle of others
    // #[test]
    fn test_it_matches_more_than_one_attribute_when_suggesting_in_the_middle() {
        let text = r##"<div hx-get="/foo" hx-target="" hx-swap="#swap">
    </div>
    "##;

        let tree = prepare_tree(&text);

        let matches = query_position(tree.root_node(), text, 1, 30);

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "".to_string()
            })
        );
    }
}
