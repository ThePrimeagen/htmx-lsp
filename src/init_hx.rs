use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HxCompletion {
    pub name: String,
    pub desc: String,
}

impl From<&(&str, &str)> for HxCompletion {
    fn from((name, desc): &(&str, &str)) -> Self {
        Self {
            name: name.to_string(),
            desc: desc.to_string(),
        }
    }
}

fn to_hx_completion(values: Vec<(&str, &str)>) -> Vec<HxCompletion> {
    values.iter().filter_map(|x| x.try_into().ok()).collect()
}

pub fn init_hx_tags() -> Vec<HxCompletion> {
    let values = vec![
        ("boost", include_str!("./md/attributes/hx-boost.md")),
        ("delete", include_str!("./md/attributes/hx-delete.md")),
        ("get", include_str!("./md/attributes/hx-get.md")),
        ("include", include_str!("./md/attributes/hx-include.md")),
        ("patch", include_str!("./md/attributes/hx-patch.md")),
        ("post", include_str!("./md/attributes/hx-post.md")),
        ("put", include_str!("./md/attributes/hx-put.md")),
        ("swap", include_str!("./md/attributes/hx-swap.md")),
        ("target", include_str!("./md/attributes/hx-target.md")),
        ("trigger", include_str!("./md/attributes/hx-trigger.md")),
        ("vals", include_str!("./md/attributes/hx-vals.md")),
        ("push-url", include_str!("./md/attributes/hx-push-url.md")),
        ("select", include_str!("./md/attributes/hx-select.md")),
        ("ext", include_str!("./md/attributes/hx-ext.md")),
        ("on", include_str!("./md/attributes/hx-on.md")),
        (
            "select-oob",
            include_str!("./md/attributes/hx-select-oob.md"),
        ),
        ("swap-oob", include_str!("./md/attributes/hx-swap-oob.md")),
        ("confirm", include_str!("./md/attributes/hx-confirm.md")),
        ("disable", include_str!("./md/attributes/hx-disable.md")),
        ("encoding", include_str!("./md/attributes/hx-encoding.md")),
        ("headers", include_str!("./md/attributes/hx-headers.md")),
        ("history", include_str!("./md/attributes/hx-history.md")),
        (
            "history-elt",
            include_str!("./md/attributes/hx-history-elt.md"),
        ),
        ("indicator", include_str!("./md/attributes/hx-indicator.md")),
        ("params", include_str!("./md/attributes/hx-params.md")),
        ("preserve", include_str!("./md/attributes/hx-preserve.md")),
        ("prompt", include_str!("./md/attributes/hx-prompt.md")),
        (
            "replace-url",
            include_str!("./md/attributes/hx-replace-url.md"),
        ),
        ("request", include_str!("./md/attributes/hx-request.md")),
        ("sync", include_str!("./md/attributes/hx-sync.md")),
        ("validate", include_str!("./md/attributes/hx-validate.md")),
    ];

    to_hx_completion(values)
}

pub fn init_hx_values() -> HashMap<String, Vec<HxCompletion>> {
    let mut hm = HashMap::new();

    let hx_swap = to_hx_completion(vec![
        ("innerHTML", include_str!("./md/hx-swap/innerHTML.md")),
        ("outerHTML", include_str!("./md/hx-swap/outerHTML.md")),
        ("afterbegin", include_str!("./md/hx-swap/afterbegin.md")),
        ("afterend", include_str!("./md/hx-swap/afterend.md")),
        ("beforebegin", include_str!("./md/hx-swap/beforebegin.md")),
        ("beforeend", include_str!("./md/hx-swap/beforeend.md")),
        ("delete", include_str!("./md/hx-swap/delete.md")),
        ("none", include_str!("./md/hx-swap/none.md")),
    ]);
    hm.insert(String::from("hx-swap"), hx_swap);

    let hx_target = to_hx_completion(vec![
        ("closest", include_str!("./md/hx-target/closest.md")),
        ("find", include_str!("./md/hx-target/find.md")),
        ("next", include_str!("./md/hx-target/next.md")),
        ("prev", include_str!("./md/hx-target/prev.md")),
        ("this", include_str!("./md/hx-target/this.md")),
    ]);
    hm.insert(String::from("hx-target"), hx_target);

    let hx_boost = to_hx_completion(vec![
        ("true", include_str!("./md/hx-boost/true.md")),
        ("false", include_str!("./md/hx-boost/false.md")),
    ]);
    hm.insert(String::from("hx-boost"), hx_boost);

    let hx_trigger = to_hx_completion(vec![
        ("click", include_str!("./md/hx-trigger/click.md")),
        ("once", include_str!("./md/hx-trigger/once.md")),
        ("changed", include_str!("./md/hx-trigger/changed.md")),
        ("delay:", include_str!("./md/hx-trigger/delay.md")),
        ("throttle:", include_str!("./md/hx-trigger/throttle.md")),
        ("from:", include_str!("./md/hx-trigger/from.md")),
        ("target:", include_str!("./md/hx-trigger/target.md")),
        ("consume", include_str!("./md/hx-trigger/consume.md")),
        ("queue:", include_str!("./md/hx-trigger/queue.md")),
        ("keyup", include_str!("./md/hx-trigger/keyup.md")),
        ("load", include_str!("./md/hx-trigger/load.md")),
        ("revealed", include_str!("./md/hx-trigger/revealed.md")),
        ("intersect", include_str!("./md/hx-trigger/intersect.md")),
        ("every", include_str!("./md/hx-trigger/every.md")),
    ]);
    hm.insert(String::from("hx-trigger"), hx_trigger);

    let hx_ext = to_hx_completion(vec![
        ("ajax-header", include_str!("./md/hx-ext/ajax-header.md")),
        ("alpine-morph", include_str!("./md/hx-ext/alpine-morph.md")),
        ("class-tools", include_str!("./md/hx-ext/class-tools.md")),
        (
            "client-side-templates",
            include_str!("./md/hx-ext/client-side-templates.md"),
        ),
        ("debug", include_str!("./md/hx-ext/debug.md")),
        (
            "disable-element",
            include_str!("./md/hx-ext/disable-element.md"),
        ),
        ("event-header", include_str!("./md/hx-ext/event-header.md")),
        ("head-support", include_str!("./md/hx-ext/head-support.md")),
        ("include-vals", include_str!("./md/hx-ext/include-vals.md")),
        ("json-enc", include_str!("./md/hx-ext/json-enc.md")),
        ("morph", include_str!("./md/hx-ext/morph.md")),
        (
            "loading-states",
            include_str!("./md/hx-ext/loading-states.md"),
        ),
        (
            "method-override",
            include_str!("./md/hx-ext/method-override.md"),
        ),
        (
            "morphdom-swap",
            include_str!("./md/hx-ext/morphdom-swap.md"),
        ),
        ("multi-swap", include_str!("./md/hx-ext/multi-swap.md")),
        ("path-deps", include_str!("./md/hx-ext/path-deps.md")),
        ("preload", include_str!("./md/hx-ext/preload.md")),
        ("remove-me", include_str!("./md/hx-ext/remove-me.md")),
        (
            "response-targets",
            include_str!("./md/hx-ext/response-targets.md"),
        ),
        ("restored", include_str!("./md/hx-ext/restored.md")),
        ("sse", include_str!("./md/hx-ext/sse.md")),
        ("ws", include_str!("./md/hx-ext/ws.md")),
    ]);
    hm.insert(String::from("hx-ext"), hx_ext);

    let hx_push_ul = to_hx_completion(vec![
        ("true", include_str!("./md/hx-push-url/true.md")),
        ("false", include_str!("./md/hx-push-url/false.md")),
    ]);
    hm.insert(String::from("hx-push-ul"), hx_push_ul);

    let hx_swap_ob = to_hx_completion(vec![
        ("true", include_str!("./md/hx-swap-oob/true.md")),
        ("innerHTML", include_str!("./md/hx-swap/innerHTML.md")),
        ("outerHTML", include_str!("./md/hx-swap/outerHTML.md")),
        ("afterbegin", include_str!("./md/hx-swap/afterbegin.md")),
        ("afterend", include_str!("./md/hx-swap/afterend.md")),
        ("beforebegin", include_str!("./md/hx-swap/beforebegin.md")),
        ("beforeend", include_str!("./md/hx-swap/beforeend.md")),
        ("delete", include_str!("./md/hx-swap/delete.md")),
        ("none", include_str!("./md/hx-swap/none.md")),
    ]);
    hm.insert(String::from("hx-swap-ob"), hx_swap_ob);

    let hx_history = to_hx_completion(vec![("false", include_str!("./md/hx-history/false.md"))]);
    hm.insert(String::from("hx-history"), hx_history);

    let hx_params = to_hx_completion(vec![
        ("*", include_str!("./md/hx-params/star.md")),
        ("none", include_str!("./md/hx-params/none.md")),
        ("not", include_str!("./md/hx-params/not.md")),
    ]);
    hm.insert(String::from("hx-params"), hx_params);

    let hx_replace_ul = to_hx_completion(vec![
        ("true", include_str!("./md/hx-replace-url/true.md")),
        ("false", include_str!("./md/hx-replace-url/false.md")),
    ]);
    hm.insert(String::from("hx-replace-ul"), hx_replace_ul);

    let hx_sync = to_hx_completion(vec![
        ("drop", include_str!("./md/hx-sync/drop.md")),
        ("abort", include_str!("./md/hx-sync/abort.md")),
        ("replace", include_str!("./md/hx-sync/replace.md")),
        ("queue", include_str!("./md/hx-sync/queue.md")),
    ]);
    hm.insert(String::from("hx-sync"), hx_sync);

    hm
}

#[derive(Debug, Clone, Copy)]
pub enum ChannelMsg {
    InitTreeSitter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LangType {
    Template,
    JavaScript,
    Backend,
}

pub enum LangTypes {
    One(LangType),
    Two { first: LangType, second: LangType },
}

impl LangTypes {
    pub fn one(lang_type: LangType) -> Self {
        Self::One(lang_type)
    }

    pub fn two(lang_types: (LangType, LangType)) -> Self {
        Self::Two {
            first: lang_types.0,
            second: lang_types.1,
        }
    }

    pub fn is_lang(&self, lang_type: LangType) -> bool {
        match self {
            LangTypes::One(lang) => lang == &lang_type,
            LangTypes::Two { first, second } => first == &lang_type || second == &lang_type,
        }
    }

    pub fn get(&self) -> LangType {
        match self {
            LangTypes::One(lang) => *lang,
            LangTypes::Two { first, .. } => *first,
        }
    }
}

impl From<usize> for LangType {
    fn from(value: usize) -> Self {
        match value {
            0 => LangType::Template,
            1 => LangType::JavaScript,
            2 => LangType::Backend,
            _ => LangType::Backend,
        }
    }
}
