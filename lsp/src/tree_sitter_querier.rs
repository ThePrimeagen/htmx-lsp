// See: https://tree-sitter.github.io/tree-sitter/using-parsers#query-syntax

use std::collections::HashMap;

use log::{debug, error};
use tree_sitter::{Node, Point, Query, QueryCursor};

use crate::tree_sitter::Position;

// If error char is "=" means the key name is completed and the cursor is
// at the "=" but no quote, so we shouldn't suggest yet eg <div hx-foo=|>
const KEY_VALUE_SEPARATOR: &str = "=";

#[derive(Debug)]
struct CaptureDetails {
    value: String,
    end_position: Point,
}

fn query_props(
    query_string: &str,
    node: Node<'_>,
    source: &str,
    trigger_point: Point,
) -> HashMap<String, CaptureDetails> {
    let query = Query::new(tree_sitter_html::language(), query_string)
        .unwrap_or_else(|_| panic!("get_position_by_query invalid query {query_string}"));
    let mut cursor_qry = QueryCursor::new();

    let capture_names = query.capture_names();

    let matches = cursor_qry.matches(&query, node, source.as_bytes());

    // Only consider the captures that are within the range based on the
    // trigger point (cursor position)
    matches
        .into_iter()
        .flat_map(|m| {
            m.captures
                .iter()
                .filter(|capture| capture.node.start_position() <= trigger_point)
        })
        .fold(HashMap::new(), |mut acc, capture| {
            let key = capture_names[capture.index as usize].to_owned();
            let value = if let Ok(capture_value) = capture.node.utf8_text(source.as_bytes()) {
                capture_value.to_owned()
            } else {
                error!("query_props capture.node.utf8_text failed {key}");
                "".to_owned()
            };

            acc.insert(
                key,
                CaptureDetails {
                    value,
                    end_position: capture.node.end_position(),
                },
            );

            acc
        })
}

pub fn query_attr_keys_for_completion(
    node: Node<'_>,
    source: &str,
    trigger_point: Point,
) -> Option<Position> {
    // [ ] means match any of the following
    let query_string = r#"
    (
        [
            (_ 
                (tag_name) 

                (_)*

                (attribute (attribute_name) @attr_name) @complete_match

                (#eq? @attr_name @complete_match)
            )

            (_ 
              (tag_name) 

              (attribute (attribute_name)) 

              (ERROR)
            ) @unfinished_tag
        ]

        (#match? @attr_name "hx-.*")
    )"#;

    let props = query_props(query_string, node, source, trigger_point);
    let attr_name = props.get("attr_name")?;

    if props.contains_key("unfinished_tag") {
        return None;
    }

    Some(Position::AttributeName(attr_name.value.to_owned()))
}

pub fn query_attr_values_for_completion(
    node: Node<'_>,
    source: &str,
    trigger_point: Point,
) -> Option<Position> {
    // [ ] means match any of the following
    let query_string = r#"(
        [
          (ERROR 
            (tag_name) 

            (attribute_name) @attr_name 
            (_)
          ) @open_quote_error

          (_ 
            (tag_name)

            (attribute 
              (attribute_name) @attr_name
              (_)
            ) @last_item

            (ERROR) @error_char
          )

          (_
            (tag_name)

            (attribute 
              (attribute_name) @attr_name
              (quoted_attribute_value) @quoted_attr_value

              (#eq? @quoted_attr_value "\"\"")
            ) @empty_attribute
          )

          (_
            (tag_name) 

            (attribute 
              (attribute_name) @attr_name
              (quoted_attribute_value (attribute_value) @attr_value)

              ) @non_empty_attribute 
          )
        ]

        (#match? @attr_name "hx-.*")
    )"#;

    let props = query_props(query_string, node, source, trigger_point);

    let attr_name = props.get("attr_name")?;

    debug!("query_attr_values_for_completion attr_name {:?}", attr_name);

    if props.contains_key("open_quote_error") || props.contains_key("empty_attribute") {
        return Some(Position::AttributeValue {
            name: attr_name.value.to_owned(),
            value: "".to_string(),
        });
    }

    if let Some(error_char) = props.get("error_char") {
        if error_char.value == KEY_VALUE_SEPARATOR {
            return None;
        }
    };

    if let Some(capture) = props.get("non_empty_attribute") {
        if trigger_point >= capture.end_position {
            return None;
        }
    }

    Some(Position::AttributeValue {
        name: attr_name.value.to_owned(),
        value: "".to_string(),
    })
}
