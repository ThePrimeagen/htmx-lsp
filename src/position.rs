use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::lsp_types::TextDocumentPositionParams;
use tree_sitter::{Node, Point};

use crate::{
    htmx_tree_sitter::LspFiles,
    init_hx::LangType,
    query_helper::{query_name, query_value, HTMLQueries, HTMLQuery},
};

#[derive(PartialEq, Eq)]
pub enum QueryType {
    Hover,
    Completion,
    Definition,
}

#[derive(Debug, Clone)]
pub struct CaptureDetails {
    pub value: String,
    pub end_position: Point,
    pub start_position: Point,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    AttributeName(String),
    AttributeValue {
        name: String,
        value: String,
        definition: Option<PositionDefinition>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionDefinition {
    pub line: usize,
    pub point: Point,
    pub start: usize,
}

impl PositionDefinition {
    pub fn new(line: usize, start: usize, point: Point) -> Self {
        Self { line, point, start }
    }
}

pub fn get_position_from_lsp_completion(
    text_params: &TextDocumentPositionParams,
    text: &DashMap<String, Rope>,
    uri: String,
    query_type: QueryType,
    lsp_files: &Arc<Mutex<LspFiles>>,
    query: &HTMLQueries,
) -> Option<Position> {
    let text = text.get(&uri)?;
    let text = text.to_string();
    let pos = text_params.position;

    if let Ok(lsp_files) = lsp_files.lock() {
        if let Some(index) = lsp_files.get_index(&uri) {
            lsp_files.query_position(index, &text, query_type, pos, query)
        } else if let Some(index) = lsp_files.add_file(String::from(&uri)) {
            lsp_files.add_tree(index, Some(LangType::Template), &text, None);
            lsp_files.query_position(index, &text, query_type, pos, query)
        } else {
            None
        }
    } else {
        None
    }
}

fn find_element_referent_to_current_node(node: Node<'_>) -> Option<Node<'_>> {
    if node.kind() == "element" || node.kind() == "fragment" {
        return Some(node);
    }

    return find_element_referent_to_current_node(node.parent()?);
}

pub fn query_position(
    root: Node<'_>,
    source: &str,
    trigger_point: Point,
    query_type: QueryType,
    query: &HTMLQueries,
) -> Option<Position> {
    let closest_node = root.descendant_for_point_range(trigger_point, trigger_point)?;
    let element = find_element_referent_to_current_node(closest_node)?;

    let name = query_name(
        element,
        source,
        trigger_point,
        &query_type,
        query.get(HTMLQuery::Name),
    );
    if name.is_some() {
        return name;
    }
    query_value(
        element,
        source,
        trigger_point,
        &query_type,
        query.get(HTMLQuery::Value),
    )
}

#[allow(dead_code)]
pub fn dbg_props(props: &HashMap<String, CaptureDetails>) {
    for i in props {
        dbg!(i);
    }
}

pub fn completion_position(props: HashMap<String, CaptureDetails>) -> Option<Position> {
    let attr_name = props.get("attr_name")?;

    if let Some(_capture) = props.get("with_attr_name_with_equals_err") {
        None
    } else if let Some(_capture) = props.get("with_attr_name_without_value_t") {
        Some(Position::AttributeName(attr_name.value.to_string()))
    } else if let Some(_capture) = props.get("with_attr_value_empty") {
        Some(Position::AttributeValue {
            name: attr_name.value.to_string(),
            value: String::new(),
            definition: None,
        })
    } else if let Some(_capture) = props.get("with_attr_value_not_empty") {
        Some(Position::AttributeValue {
            name: attr_name.value.to_string(),
            value: String::new(),
            definition: None,
        })
    } else {
        props
            .get("with_error_with_value_t_no_second_quote")
            .map(|_capture| Position::AttributeValue {
                name: attr_name.value.to_string(),
                value: String::new(),
                definition: None,
            })
    }
}

pub fn hover_position(
    props: HashMap<String, CaptureDetails>,
    client_point: Point,
) -> Option<Position> {
    let attr_name = props.get("attr_name")?;
    if let Some(capture) = props.get("with_attr_value_not_empty") {
        if client_point > capture.end_position {
            return None;
        }
        let attr_value = props.get("attr_value");
        if let Some(capture) = attr_value {
            if client_point >= attr_name.end_position {
                return Some(Position::AttributeValue {
                    name: attr_name.value.to_string(),
                    value: capture.value.to_string(),
                    definition: None,
                });
            }
        }
        if client_point <= attr_name.end_position {
            return Some(Position::AttributeName(attr_name.value.to_string()));
        }
        None
        // Some(MyPosition::AttributeValue {
        //     name: attr_name.value.to_string(),
        //     value: attr_value.value.to_string(),
        // })
    } else if let Some(capture) = props.get("with_attr_value_empty") {
        if client_point > capture.end_position {
            return None;
        }
        let attr_value = props.get("attr_value");
        match attr_value {
            Some(capture) => Some(Position::AttributeValue {
                name: attr_name.value.to_string(),
                value: capture.value.to_string(),
                definition: None,
            }),
            None => Some(Position::AttributeName(attr_name.value.to_string())),
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests1 {
    use tree_sitter::{Parser, Point};

    use crate::{
        position::{query_position, Position, QueryType},
        query_helper::{query_props, HTMLQueries, Queries},
    };

    fn prepare_tree(text: &str) -> tree_sitter::Tree {
        let language = tree_sitter_html::language();
        let mut parser = Parser::new();

        parser
            .set_language(language)
            .expect("could not load html grammer");

        parser.parse(text, None).expect("not to fail")
    }

    fn prepare_python_tree(text: &str) -> tree_sitter::Tree {
        let language = tree_sitter_python::language();
        let mut parser = Parser::new();

        parser
            .set_language(language)
            .expect("could not load html grammer");

        parser.parse(text, None).expect("not to fail")
    }

    #[test]
    fn suggests_attr_names_when_starting_tag() {
        let text = r##"<div hx- ></div>"##;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 8),
            QueryType::Completion,
            &query,
        );
        // // Fixes issue with not suggesting hx-* attributes
        // let expected = get_position(tree.root_node(), text, 0, 8);
        // assert_eq!(matches, expected);
        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn does_not_suggest_when_quote_not_initiated() {
        let text = r##"<div hx-swap= ></div>"##;

        let tree = prepare_tree(text);

        // let expected = get_position(tree.root_node(), text, 0, 13);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 13),
            QueryType::Completion,
            &query,
        );

        // assert_eq!(matches, expected);
        assert_eq!(matches, None);
    }

    #[test]
    fn suggests_attr_values_when_starting_quote_value() {
        let text = r#"<div hx-swap=" ></div>"#;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 14),
            QueryType::Completion,
            &query,
        );

        // The new implementation doesn't return incomplete tags as value :)
        // let expected = get_position(tree.root_node(), text, 0, 14);
        // assert_eq!(matches, expected);
        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-swap".to_string(),
                value: "".to_string(),
                definition: None
            })
        );
    }

    #[test]
    fn suggests_attr_values_when_open_and_closed_quotes() {
        let text = r#"<div hx-swap=""></div>"#;
        // <div hx-swap=""></div>

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 13),
            QueryType::Completion,
            &query,
        );

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-swap".to_string(),
                value: "".to_string(),
                definition: None
            })
        );
    }

    #[test]
    fn suggests_attr_values_once_opening_quotes_in_between_tags() {
        let text = r#"<div id="fa" hx-swap="hx-swap" hx-swap="hx-swap">
      <span hx-target="
      <button>Click me</button>
    </div>
    "#;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(1, 23),
            QueryType::Completion,
            &query,
        );

        // The new implementation doesn't return incomplete tags as value :)
        // let expected = get_position(tree.root_node(), text, 1, 16);
        // assert_eq!(matches, expected);
        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "".to_string(),
                definition: None
            })
        );
    }

    #[test]
    fn suggests_attr_names_for_incomplete_attr_in_between_tags() {
        let text = r#"<div id="fa" hx-target="this" hx-swap="hx-swap">
      <span hx-
      <button>Click me</button>
    </div>
    "#;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(1, 14),
            QueryType::Completion,
            &query,
        );

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn matches_more_than_one_attribute() {
        let text = r#"<div hx-get="/foo" hx-target="this" hx- ></div>"#;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 39),
            QueryType::Completion,
            &query,
        );

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn suggests_attr_value_when_attr_is_empty_and_in_between_attributes() {
        let text = r##"<div hx-get="/foo" hx-target="" hx-swap="#swap"></div>
    "##;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 30),
            QueryType::Completion,
            &query,
        );

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "".to_string(),
                definition: None
            })
        );
    }

    #[test]
    fn suggests_attr_values_for_incoplete_quoted_attr_when_in_between_attributes() {
        let text = r##"<div hx-get="/foo" hx-target=" hx-swap="#swap"></div>"##;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 30),
            QueryType::Completion,
            &query,
        );

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "".to_string(),
                definition: None
            })
        );
    }

    #[test]
    fn suggests_attr_names_for_incoplete_quoted_value_in_between_attributes() {
        let text = r##"<div hx-get="/foo" hx- hx-swap="#swap"></div>
        <span class="foo" />"##;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 22),
            QueryType::Completion,
            &query,
        );

        assert_eq!(matches, Some(Position::AttributeName("hx-".to_string())));
    }

    #[test]
    fn suggests_attribute_keys_when_half_completeded() {
        let text = r##"<div hx-get="/foo" hx-t hx-swap="#swap"></div>
        <span class="foo" />"##;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 23),
            QueryType::Completion,
            &query,
        );

        assert_eq!(matches, Some(Position::AttributeName("hx-t".to_string())));
    }

    #[test]
    fn suggests_values_for_already_filled_attributes() {
        let text = r##"<div hx-get="/foo" hx-target="find " hx-swap="#swap"></div>"##;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 35),
            QueryType::Hover,
            &query,
        );

        assert_eq!(
            matches,
            Some(Position::AttributeValue {
                name: "hx-target".to_string(),
                value: "find ".to_string(),
                definition: None
            })
        );
    }

    #[test]
    fn does_not_suggest_when_cursor_isnt_within_a_htmx_attribute() {
        let text = r#"<div hx-get="/foo"  class="p-4" ></div>"#;

        let tree = prepare_tree(text);

        let query = HTMLQueries::default();
        let matches = query_position(
            tree.root_node(),
            text,
            Point::new(0, 24),
            QueryType::Hover,
            &query,
        );

        assert_eq!(matches, None);
    }

    #[test]
    fn hover_hx_tags() {
        let cases = [
            (
                r#"<div hx-get="/foo" class="p-4" hx-target="closest" ></div>"#,
                Point::new(0, 37),
                Some(Position::AttributeName(String::from("hx-target"))),
            ),
            (
                r#"<div hx-get="" class="p-4" hx-target="" ></div>"#,
                Point::new(0, 9),
                Some(Position::AttributeName(String::from("hx-get"))),
                // None,
            ),
            (
                r#"<div hx-get="/foo" hx-target="closest" hx-swap="outerHTML" hx-swap="swap"></div>"#,
                Point::new(0, 9),
                Some(Position::AttributeName(String::from("hx-get"))),
            ),
            (
                r#"<a hx-swap="" hx-patch="/route" hx-validate"#,
                Point::new(0, 40),
                Some(Position::AttributeName(String::from("hx-validate"))),
            ),
        ];

        for case in cases {
            let text = case.0;
            let tree = prepare_tree(text);
            let query = HTMLQueries::default();
            let matches = query_position(tree.root_node(), text, case.1, QueryType::Hover, &query);
            assert_eq!(matches, case.2);
        }
    }

    #[test]
    fn unfinished_tag_name() {
        let cases = [(
            r#"<a hx-swap class="text-2xl">
       
</a>
                
            "#,
            Point::new(1, 5),
            QueryType::Completion,
        )];
        for case in cases {
            let text = case.0;
            let tree = prepare_tree(text);
            let query = HTMLQueries::default();
            let matches = query_position(tree.root_node(), text, case.1, case.2, &query);
            assert_eq!(matches, Some(Position::AttributeName(String::from("--"))));
            // assert_eq!(matches, case.2);
        }
    }

    #[test]
    fn python_tags() {
        let case = r#"
def a():
    # hx@hello
    # hx@world
    print("hello world")
    # hx@hello_world
        "#;
        let tree = prepare_python_tree(case);
        let trigger_point = Point::new(0, 0);
        let closest_node = tree.root_node();
        let mut query = Queries::default();
        query.change_backend("python");
        let query = &query.backend;
        let props = query_props(closest_node, case, trigger_point, query, true);
        assert_eq!(props.len(), 3);
    }
}

// pub static QUICK_QUERY: &'static str = r#""#;

pub static QUICK_QUERY: &str = r#"
(
  [

    (_
      (tag_name)
      (_)*
      (attribute
          (attribute_name) @attr_name
          (quoted_attribute_value
          	(attribute_value) @attr_value
            (_)*
          ) @quoted_value

      ) @with_attr_value_not_empty
    )

    (_

      (tag_name)
        (attribute
          (attribute_name) @attr_name)
		(ERROR)
       @with_attr_name_with_equals_err
    )

    (_
      (tag_name)
      (attribute
          (attribute_name) @attr_name
          (quoted_attribute_value) @quoted_attr_value

          (#eq? @quoted_attr_value "\"\"")

      ) @with_attr_value_empty
    )

    (_
      (ERROR
      (tag_name)

          (attribute_name) @attr_name
          (attribute_value) @attr_value

       @with_error_with_value_t_no_second_quote
      )
    )

    (_

      (tag_name)
        (attribute
          (attribute_name) @attr_name
          )

       @with_attr_name_without_value_t
       (#eq? @attr_name @with_attr_name_without_value_t)

    )

    (_
      (tag_name)
      (attribute
          (attribute_name) @attr_name
          (attribute_value) @attr_value

      ) @no_second_quote
    )

  ]

	(#match? @attr_name "hx-.*")
)

"#;
