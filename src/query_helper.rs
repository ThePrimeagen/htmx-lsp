use std::collections::HashMap;

use tree_sitter::{Node, Point, Query, QueryCursor};

use crate::{
    htmx_tags::{get_tag, get_tags, Tag},
    init_hx::LangType,
    position::{CaptureDetails, Position, PositionDefinition, QueryType},
    queries::{
        HX_ANY_HTML, HX_GO_TAGS, HX_HTML, HX_JS_TAGS, HX_NAME, HX_PYTHON_TAGS, HX_RUST_TAGS,
        HX_VALUE,
    },
};

/// Container for all queries. This struct can be cloned and used in other threads.
/// It doesn't contain state about previous results of query.
pub struct Queries {
    /// Check `HTMLQueries` for more info.
    pub html: HTMLQueries,
    /// JavaScript/TypeScript query. TreeSitter query for both languages is same.
    pub javascript: Query,
    /// Backend tags query. Can be in Python, Rust, Go.
    pub backend: Query,
}

impl Clone for Queries {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Default for Queries {
    fn default() -> Self {
        Self {
            html: HTMLQueries::default(),
            javascript: Query::new(tree_sitter_javascript::language(), HX_JS_TAGS).unwrap(),
            backend: Query::new(tree_sitter_rust::language(), HX_RUST_TAGS).unwrap(),
        }
    }
}

impl Queries {
    /// Get query.
    pub fn get(&self, query: HtmxQuery) -> &Query {
        match query {
            HtmxQuery::Html(html) => self.html.get(html),
            HtmxQuery::JavaScript => &self.javascript,
            HtmxQuery::Backend => &self.backend,
        }
    }

    /// Default backend language is Rust. Change at the beginning to other.
    pub fn change_backend(&mut self, lang: &str) -> Option<()> {
        let lang = match lang {
            "python" => Some((tree_sitter_python::language(), HX_PYTHON_TAGS)),
            "go" => Some((tree_sitter_go::language(), HX_GO_TAGS)),
            _ => None,
        };
        if let Some(lang) = lang {
            self.backend = Query::new(lang.0, lang.1).unwrap();
        }
        None
    }
}

/// HTMLQueries has three queries:
/// * lsp `HX_HTML`
/// * name `HX_NAME`
/// * value `HX_VALUE`   
pub struct HTMLQueries {
    lsp: Query,
    name: Query,
    value: Query,
}

impl Default for HTMLQueries {
    fn default() -> Self {
        let lsp = Query::new(tree_sitter_html::language(), HX_HTML).unwrap();
        let name = Query::new(tree_sitter_html::language(), HX_NAME).unwrap();
        let value = Query::new(tree_sitter_html::language(), HX_VALUE).unwrap();
        Self { lsp, name, value }
    }
}

impl Clone for HTMLQueries {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl HTMLQueries {
    pub fn get(&self, query: HTMLQuery) -> &Query {
        match query {
            HTMLQuery::Lsp => &self.lsp,
            HTMLQuery::Name => &self.name,
            HTMLQuery::Value => &self.value,
        }
    }

    /// Generate new query, for some random, non-htmx attribute.
    pub fn get_by_attribute_name(name: &str) -> Query {
        Query::new(
            tree_sitter_html::language(),
            &HX_ANY_HTML.replace("NAME", name),
        )
        .unwrap()
    }
}

/// HTML can have multiple query types.
pub enum HTMLQuery {
    Lsp,
    Name,
    Value,
}

/// HtmxQuery
pub enum HtmxQuery {
    Html(HTMLQuery),
    JavaScript,
    Backend,
}

impl TryFrom<LangType> for HtmxQuery {
    type Error = ();

    fn try_from(value: LangType) -> Result<Self, Self::Error> {
        match value {
            LangType::Template => Err(()),
            LangType::JavaScript => Ok(HtmxQuery::JavaScript),
            LangType::Backend => Ok(HtmxQuery::Backend),
        }
    }
}

/// Capture all query results. No duplicates, except when searching for hx_comment.
pub fn query_props(
    node: Node<'_>,
    source: &str,
    trigger_point: Point,
    query: &Query,
    all: bool,
) -> HashMap<String, CaptureDetails> {
    let mut cursor_qry = QueryCursor::new();
    let capture_names = query.capture_names();
    let matches = cursor_qry.matches(query, node, source.as_bytes());

    let mut cnt = 0;
    matches
        .into_iter()
        .flat_map(|m| {
            m.captures
                .iter()
                .filter(|capture| all || capture.node.start_position() <= trigger_point)
        })
        .fold(HashMap::new(), |mut acc, capture| {
            let key = capture_names[capture.index as usize].to_owned();
            let value = if let Ok(capture_value) = capture.node.utf8_text(source.as_bytes()) {
                capture_value.to_owned()
            } else {
                "".to_owned()
            };
            if key == "hx_comment" {
                cnt += 1;
            }
            let key = {
                if all {
                    format!("{}{cnt}", key)
                } else {
                    key
                }
            };

            acc.insert(
                key,
                CaptureDetails {
                    value,
                    end_position: capture.node.end_position(),
                    start_position: capture.node.start_position(),
                },
            );

            acc
        })
}

/// Query only attribute name. Can be used in testing.
pub fn query_name(
    element: Node<'_>,
    source: &str,
    trigger_point: Point,
    query_type: &QueryType,
    query: &Query,
) -> Option<Position> {
    let props = query_props(element, source, trigger_point, query, false);
    let attr_name = props.get("attr_name")?;
    if let Some(unfinished_tag) = props.get("unfinished_tag") {
        if query_type == &QueryType::Hover {
            let complete_match = props.get("complete_match");
            if complete_match.is_some() && trigger_point <= attr_name.end_position {
                return Some(Position::AttributeName(attr_name.value.to_string()));
            }
            return None;
        } else if query_type == &QueryType::Completion
            && trigger_point > unfinished_tag.end_position
        {
            return Some(Position::AttributeName(String::from("--")));
        } else if let Some(_capture) = props.get("equal_error") {
            if query_type == &QueryType::Completion {
                return None;
            }
        }
    }

    Some(Position::AttributeName(attr_name.value.to_string()))
}

/// Query for attribute values. Can be used for testing.
pub fn query_value(
    element: Node<'_>,
    source: &str,
    trigger_point: Point,
    query_type: &QueryType,
    query: &Query,
) -> Option<Position> {
    let props = query_props(element, source, trigger_point, query, false);

    let attr_name = props.get("attr_name")?;
    let mut value = String::new();
    let mut definition = None;
    let hovered_name = trigger_point < attr_name.end_position && query_type == &QueryType::Hover;
    if hovered_name {
        return Some(Position::AttributeName(attr_name.value.to_string()));
    } else if props.get("open_quote_error").is_some() || props.get("empty_attribute").is_some() {
        if query_type == &QueryType::Completion {
            if let Some(quoted) = props.get("quoted_attr_value") {
                if trigger_point >= quoted.end_position {
                    return None;
                }
            }
        }
        return Some(Position::AttributeValue {
            name: attr_name.value.to_owned(),
            value: "".to_string(),
            definition: None,
        });
    }

    if let Some(error_char) = props.get("error_char") {
        if error_char.value == "=" {
            return None;
        }
    };

    if let Some(capture) = props.get("non_empty_attribute") {
        if trigger_point >= capture.end_position {
            return None;
        }
        if query_type == &QueryType::Hover || query_type == &QueryType::Definition {
            let mut start = 0;
            let _ = props.get("attr_value").is_some_and(|s| {
                value = s.value.to_string();
                start = s.start_position.column;
                true
            });
            if query_type == &QueryType::Definition {
                //
                definition = Some(PositionDefinition::new(start, trigger_point));
            }
        }
    }

    Some(Position::AttributeValue {
        name: attr_name.value.to_owned(),
        value,
        definition,
    })
}

/// Query for htmx tags on backend/javascript.
pub fn query_tag(
    element: Node<'_>,
    source: &str,
    trigger_point: Point,
    _query_type: &QueryType,
    query: &Query,
    full: bool,
) -> Vec<Tag> {
    let comments = query_props(element, source, trigger_point, query, full);
    let mut tags = vec![];
    for comment in comments {
        if let Some(mut tag) = get_tag(&comment.1.value) {
            tag.start.row = comment.1.start_position.row;
            tag.end.row = comment.1.start_position.row;
            tags.push(tag);
        }
    }
    tags
}

/// Capture all tags(if any is found) that matches `tag_name` parameter.
#[allow(clippy::too_many_arguments)]
pub fn query_htmx_lsp(
    element: Node<'_>,
    source: &str,
    trigger_point: Point,
    _query_type: &QueryType,
    query: &Query,
    tag_name: &str,
    references: &mut Vec<Tag>,
    file: usize,
) {
    let lsp_names = query_props(element, source, trigger_point, query, true);
    for capture in lsp_names {
        if capture.0.starts_with("attr_value") {
            let value = capture.1.value;
            let tags = get_tags(
                &value,
                capture.1.start_position.column,
                capture.1.start_position.row,
            );
            if let Some(tags) = tags {
                let tag = tags.iter().find(|item| item.name == tag_name);
                if let Some(tag) = tag {
                    let mut tag = tag.clone();
                    tag.file = file;
                    references.push(tag);
                }
            }
            // let position = PositionDefinition::new(capture.1.start_position.row, capture.1.start_position.column);
            //
        }
    }
}

/// `HX_HTML`
pub fn find_hx_lsp(
    element: Node<'_>,
    source: String,
    trigger_point: Point,
    query: &Query,
) -> Option<CaptureDetails> {
    let props = query_props(element, &source, trigger_point, query, false);
    if props.get("attr_name").is_some() {
        let value = props.get("attr_value")?;
        return Some(value.clone());
    }
    None
}
