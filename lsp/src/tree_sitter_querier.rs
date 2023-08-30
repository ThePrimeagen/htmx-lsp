// See: https://tree-sitter.github.io/tree-sitter/using-parsers#query-syntax

use std::collections::HashMap;

use tree_sitter::{Node, Point, Query, QueryCursor};

use crate::tree_sitter::Position;

fn query_props(
    query_string: &str,
    node: Node<'_>,
    source: &str,
    trigger_point: Point,
) -> Option<HashMap<String, String>> {
    let query = Query::new(tree_sitter_html::language(), query_string).expect(&format!(
        "get_position_by_query invalid query {query_string}"
    ));
    let mut cursor_qry = QueryCursor::new();
    let mut matches = cursor_qry.matches(&query, node, source.as_bytes());
    let capture_names = query.capture_names();

    let mut props = HashMap::new();
    let match_ = matches.next()?;
    match_.captures.iter().for_each(|capture| {
        let name = capture_names[capture.index as usize].to_owned();
        let value = capture
            .node
            .utf8_text(source.as_bytes())
            .expect(&format!("failed to parse capture value for '{name}'"))
            .to_owned();
        props.insert(name, value);
    });

    Some(props)
}

pub fn query_attributes_for_completion(
    node: Node<'_>,
    source: &str,
    trigger_point: Point,
) -> Option<Position> {
    let query_string = r#"
    (
        (_
            [
                (_
                    (tag_name)

                    (attribute (attribute_name) (quoted_attribute_value (attribute_value)))*
                    (attribute (attribute_name) @attr_name) @attribute

                    (ERROR) @error_char
                )
                (_ 
                    (tag_name) 
                    (_)*
                    (attribute (attribute_name) @attr_name) @attribute
                    (#eq? @attr_name "hx-")
                )
            ]
        )
        (#match? @attr_name "hx-.*=?")
    )"#;

    let attr_completion = query_props(query_string, node, source, trigger_point);
    let props = attr_completion?;
    let attr_name = props.get("attr_name")?;
    if props.get("error_char").is_some() {
        let error_char = props.get("error_char")?;
        if error_char == "=" {
            return None;
        }
    }
    return Some(Position::AttributeName(attr_name.to_owned()));
}

pub fn query_attr_values_for_completion(
    node: Node<'_>,
    source: &str,
    trigger_point: Point,
) -> Option<Position> {
    let query_string = r#"(
      (_
        [
          (ERROR 
            (tag_name) 
            (attribute_name) @attr_name 
            (attribute_value) @attr_value
          ) @open_quote_err

          (_
            (tag_name)

            (attribute (attribute_name) (quoted_attribute_value (attribute_value)))*
            (attribute (attribute_name) @attr_name) @attribute

            (ERROR) @error_char
          )

          (_
            (tag_name)

            (attribute 
              (attribute_name) @attr_name
              (quoted_attribute_value) @quoted_attr_value
              (#eq? @quoted_attr_value "\"\"")
            ) @empty_quoted_value
          )
        ]
      )
    )"#;

    let value_completion = query_props(query_string, node, source, trigger_point);
    let props = value_completion?;
    let attr_name = props.get("attr_name")?;
    if props.get("open_quote_err").is_some() || props.get("empty_quoted_value").is_some() {
        return Some(Position::AttributeValue {
            name: attr_name.to_string(),
            value: "".to_string(),
        });
    }

    if let Some(error_char) = props.get("error_char") {
        if error_char == "=" {
            return None;
        }

        return Some(Position::AttributeValue {
            name: attr_name.to_string(),
            value: "".to_string(),
        });
    } else {
        return Some(Position::AttributeValue {
            name: attr_name.to_string(),
            value: "".to_string(),
        });
    }
}
