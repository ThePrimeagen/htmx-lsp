use log::error;
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
    let query = Query::new(tree_sitter_html::language(), query).unwrap();

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, node, source.as_bytes());

    println!("-- node {:?}", node.to_sexp());

    let formatted = matches
        .map(|m| {
            println!("--!!!! match {:?}", m);
            let mut values: HashMap<String, String> = HashMap::new();
            let captures = m.captures;
            println!("---- << captures {:?}", captures.to_vec());
            for capture in captures.iter() {
                println!("-- << capture {:?}", capture);
                let capture_name = query.capture_names()[capture.index as usize].as_str();
                println!("-- << capture_name {:?}", capture_name);

                let value = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap()
                    .to_string();

                values.insert(capture_name.to_string(), value);
            }
            println!("-- << values {:?}", values);
            values
        })
        .map(|v| match v.get("attr_value") {
            Some(value) => Some(Position::AttributeValue {
                name: format!("{}", "attr_name"),
                value: format!("{}", value),
            }),
            None => match v.get("attr_name") {
                Some(name) => Some(Position::AttributeName(format!("{}", name))),
                None => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();

    return formatted.first()?.clone();
}

fn find_start_tag_node(node: Node<'_>) -> Option<Node<'_>> {
    println!("-- node {:?}", node.to_sexp());
    if node.kind() == "element" || node.kind() == "fragment" {
        return Some(node.child(0)?);
    }

    let parent = node.parent()?;

    if parent.kind() == "element" {
        return Some(node);
    }

    if node.kind() == "ERROR" {
        return Some(node);
    }

    return find_start_tag_node(parent);
}

fn query_position(root: Node<'_>, source: &str, row: usize, column: usize) -> Option<Position> {
    error!("get_position");

    println!("-- root {:?}", root.to_sexp());
    let desc = root.descendant_for_point_range(Point { row, column }, Point { row, column })?;
    println!("-- desc {:?}", desc.to_sexp());
    let node = find_start_tag_node(desc)?;
    println!("------ FOUND node {:?}", node.to_sexp());

    if node.kind() == "ERROR" {
        return get_position_by_query(
            r#"
    (
        (ERROR 
            (tag_name)
            (attribute_name) @attr_name
        )
        (#match? @attr_name "hx-.*?")
    )
    "#,
            node,
            source,
        );
    }

    return get_position_by_query(
        r#"
    (
        (start_tag 
            (tag_name)
            (attribute (attribute_name) @attr_name
                (quoted_attribute_value 
                    (attribute_value)? @attr_value
                )? @quoted_attr_value
            )
        ) @tag
        (#match? @attr_name "hx-.*?")
    )
"#,
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

    return get_position(
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

    #[test]
    fn test_it_matches_when_starting_tag() {
        let text = r##"<div hx- ></div>"##;
        let language = tree_sitter_html::language();
        let mut parser = Parser::new();

        parser
            .set_language(language)
            .expect("could not load html grammer");
        let tree = parser.parse(&text, None).expect("not to fail");
        let matches = query_position(tree.root_node(), text, 0, 4);

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn test_it_matches_when_completing_with_values() {
        let text = r##"<div hx-swap=></div>"##;
        let language = tree_sitter_html::language();
        let mut parser = Parser::new();

        parser
            .set_language(language)
            .expect("could not load html grammer");
        let tree = parser.parse(&text, None).expect("not to fail");
        let matches = query_position(tree.root_node(), text, 0, 13);

        assert_eq!(
            matches,
            Some(Position::AttributeName("hx-swap".to_string()))
        );
    }

    #[test]
    fn test_it_matches_when_starting_quote_value() {
        let text = r##"<div hx-swap="></div>"##;
        let language = tree_sitter_html::language();
        let mut parser = Parser::new();

        parser
            .set_language(language)
            .expect("could not load html grammer");
        let tree = parser.parse(&text, None).expect("not to fail");
        let matches = get_position(tree.root_node(), text, 0, 14);

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-swap".to_string(),
                value: "></div>".to_string()
            })
        );
    }
    //
    // #[test]
    // fn test_it_matches_when_open_and_closed_quotes() {
    //     let text = r##"<div hx-swap=""></div>"##;
    //     let language = tree_sitter_html::language();
    //     let mut parser = Parser::new();
    //
    //     parser
    //         .set_language(language)
    //         .expect("could not load html grammer");
    //     let tree = parser.parse(&text, None).expect("not to fail");
    //     let matches = query_position(tree.root_node(), text, 0, 14);
    //
    //     assert_eq!(
    //         matches,
    //         Some(Position::AttributeValue {
    //             name: "hx-swap".to_string(),
    //             value: "\"\"".to_string()
    //         })
    //     );
    // }
    //
    // #[test]
    // fn test_it_matches_a_unclosed_tag_in_the_middle() {
    //     let text = r##"<div id="fa" hx-swap="hx-swap" hx-swap="hx-swap">
    //   <span hx-swap="
    //   <button>Click me</button>
    // </div>
    // "##;
    //     let language = tree_sitter_html::language();
    //     let mut parser = Parser::new();
    //
    //     parser
    //         .set_language(language)
    //         .expect("could not load html grammer");
    //     let tree = parser.parse(&text, None).expect("not to fail");
    //     let matches = query_position(tree.root_node(), text, 1, 17);
    //
    //     assert_eq!(
    //         matches,
    //         Some(Position::AttributeValue {
    //             name: "hx-swap".to_string(),
    //             value: "\n  <button>Click me</button>\n</div>\n".to_string()
    //         })
    //     );
    // }
    //
    // #[test]
    // fn test_error() {
    //     let text = r##"<div hx-"##;
    //     let language = tree_sitter_html::language();
    //     let mut parser = Parser::new();
    //
    //     parser
    //         .set_language(language)
    //         .expect("could not load html grammer");
    //     let tree = parser.parse(&text, None).expect("not to fail");
    //     let matches = query_position(tree.root_node(), text, 1, 7);
    //
    //     assert_eq!(
    //         matches,
    //         Some(Position::AttributeValue {
    //             name: "hx-swap".to_string(),
    //             value: "\n  <button>Click me</button>\n</div>\n".to_string()
    //         })
    //     );
    // }
    //
    //     #[test]
    //     fn test_it_matches_a_unclosed_tag_in_the_middle_() {
    //         let text = r##"<div id="fa" hx-swap="hx-swap" hx-swap="hx-swap">
    //   <span hx-
    //   <tebutton>Click me</button>
    // </div>
    // "##;
    //         let language = tree_sitter_html::language();
    //         let mut parser = Parser::new();
    //
    //         parser
    //             .set_language(language)
    //             .expect("could not load html grammer");
    //         let tree = parser.parse(&text, None).expect("not to fail");
    //         let matches = query_position(tree.root_node(), text, 1, 11);
    //
    //         assert_eq!(
    //             matches,
    //             Some(Position::AttributeValue {
    //                 name: "hx-swap".to_string(),
    //                 value: "\n  <tebutton>Click me</button>\n</div>\n".to_string()
    //             })
    //         );
    //     }
}
