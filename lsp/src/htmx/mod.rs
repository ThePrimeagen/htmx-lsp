use std::{sync::OnceLock, path::PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HxAttribute {
    pub name: String,
    pub desc: String,
}

impl From<&(&str, &str)> for HxAttribute {
    fn from((name, desc): &(&str, &str)) -> Self {
        Self {
            name: name.to_string(),
            desc: desc.to_string(),
        }
    }
}

impl TryFrom<&(PathBuf, String)> for HxAttribute {
    type Error = anyhow::Error;

    fn try_from((path, desc): &(PathBuf, String)) -> Result<Self, Self::Error> {
        let name = path.to_str().unwrap_or("").to_string();
        if name == "" {
            return Err(anyhow::anyhow!("Invalid path"));
        }
        return Ok(Self {
            name,
            desc: desc.to_string(),
        });
    }
}

pub static HX_TAGS: OnceLock<Vec<HxAttribute>> = OnceLock::new();

pub fn init_hx_tags() {
    _ = HX_TAGS.set(
        vec![
            ("hx-boost", include_str!("./attributes/hx-boost.md")),
            ("hx-delete", include_str!("./attributes/hx-delete.md")),
            ("hx-get", include_str!("./attributes/hx-get.md")),
            ("hx-include", include_str!("./attributes/hx-include.md")),
            ("hx-patch", include_str!("./attributes/hx-patch.md")),
            ("hx-post", include_str!("./attributes/hx-post.md")),
            ("hx-put", include_str!("./attributes/hx-put.md")),
            ("hx-swap", include_str!("./attributes/hx-swap.md")),
            ("hx-target", include_str!("./attributes/hx-target.md")),
            ("hx-trigger", include_str!("./attributes/hx-trigger.md")),
            ("hx-vals", include_str!("./attributes/hx-vals.md")),
            ("hx-push-url", include_str!("./attributes/hx-push-url.md")),
            ("hx-select", include_str!("./attributes/hx-select.md")),
        ]
            .iter()
            .filter_map(|x| x.try_into().ok())
            .collect()
    );

    /*
    _ = HX_TAGS.set(vec![
        ("hx-get", "GET the resource at the provided URL and replaces the element with the hx-swap strategy."),
        ("hx-post", "POST the element's form data, hx-include, and hx-vals to the URL."),
        ("hx-delete", "Sends a DELETE request to the URL."),
        ("hx-put", "Sends a PUT request to the URL."),
        ("hx-patch", "Sends a PATCH request to the URL."),
        ("hx-vals", "Submits the element's form data to the URL, but only the named fields."),
        ("hx-include", "Fetches the resource at the given URL and inserts the response into the element."),
        ("hx-swap", "Fetches the resource at the given URL and replaces the element with the response, but only the named fields."),
        ("hx-target", "Fetches the resource at the given URL and replaces the element with the response, but only the named fields."),
        ("hx-boost", "Fetches the resource at the given URL and replaces the element with the response, but only the named fields."),
    ].iter().map(|x| x.into()).collect());
    */
}



