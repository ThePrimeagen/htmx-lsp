use log::debug;
use lsp_types::TextDocumentPositionParams;
use serde::{Deserialize, Serialize};

use crate::tree_sitter::Position;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HxCompletion {
    pub name: &'static str,
    pub desc: &'static str,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HxHover {
    pub name: String,
    pub desc: String,
}

macro_rules! build_completion {
    ($(($name:expr, $desc:expr)),*) => {
        &[
            $(HxCompletion {
            name: $name,
            desc: include_str!($desc),
            }),*
        ]
    };
}

pub fn hx_completion(text_params: TextDocumentPositionParams) -> Option<&'static [HxCompletion]> {
    let result = crate::tree_sitter::get_position_from_lsp_completion(text_params.clone())?;

    debug!("result: {:?} params: {:?}", result, text_params);

    match result {
        Position::AttributeName(name) => {
            if name.starts_with("hx-") {
                return Some(HX_TAGS);
            }
        }

        Position::AttributeValue { name, .. } => {
            let values = HX_ATTRIBUTE_VALUES.get(&name)?;
            let completions = match values {
                VariadicCompletion::HxSwap(v) => v.as_slice(),
                VariadicCompletion::HxTarget(v) => v.as_slice(),
                VariadicCompletion::HxBoost(v) => v.as_slice(),
                VariadicCompletion::HxExt(v) => v.as_slice(),
                VariadicCompletion::HxTrigger(v) => v.as_slice(),
                VariadicCompletion::HxPushUrl(v) => v.as_slice(),
                VariadicCompletion::HxSwapOob(v) => v.as_slice(),
                VariadicCompletion::HxHistory(v) => v.as_slice(),
                VariadicCompletion::HxParams(v) => v.as_slice(),
                VariadicCompletion::HxReplaceUrl(v) => v.as_slice(),
                VariadicCompletion::HxSync(v) => v.as_slice(),
            };
            return Some(completions);
        }
    };

    None
}

pub fn hx_hover(text_params: TextDocumentPositionParams) -> Option<HxCompletion> {
    let result = crate::tree_sitter::get_position_from_lsp_completion(text_params.clone())?;
    debug!("handle_hover result: {:?}", result);

    match result {
        Position::AttributeName(name) => HX_TAGS.iter().find(|x| x.name == name).cloned(),
        Position::AttributeValue { name, .. } => HX_TAGS.iter().find(|x| x.name == name).cloned(),
    }
}

pub static HX_TAGS: &[HxCompletion] = build_completion!(
    ("hx-boost", "./attributes/hx-boost.md"),
    ("hx-delete", "./attributes/hx-delete.md"),
    ("hx-get", "./attributes/hx-get.md"),
    ("hx-include", "./attributes/hx-include.md"),
    ("hx-patch", "./attributes/hx-patch.md"),
    ("hx-post", "./attributes/hx-post.md"),
    ("hx-put", "./attributes/hx-put.md"),
    ("hx-swap", "./attributes/hx-swap.md"),
    ("hx-target", "./attributes/hx-target.md"),
    ("hx-trigger", "./attributes/hx-trigger.md"),
    ("hx-vals", "./attributes/hx-vals.md"),
    ("hx-push-url", "./attributes/hx-push-url.md"),
    ("hx-select", "./attributes/hx-select.md"),
    ("hx-ext", "./attributes/hx-ext.md"),
    ("hx-on", "./attributes/hx-on.md"),
    ("hx-select-oob", "./attributes/hx-select-oob.md"),
    ("hx-swap-oob", "./attributes/hx-swap-oob.md"),
    ("hx-confirm", "./attributes/hx-confirm.md"),
    ("hx-disable", "./attributes/hx-disable.md"),
    ("hx-encoding", "./attributes/hx-encoding.md"),
    ("hx-headers", "./attributes/hx-headers.md"),
    ("hx-history", "./attributes/hx-history.md"),
    ("hx-history-elt", "./attributes/hx-history-elt.md"),
    ("hx-indicator", "./attributes/hx-indicator.md"),
    ("hx-params", "./attributes/hx-params.md"),
    ("hx-preserve", "./attributes/hx-preserve.md"),
    ("hx-prompt", "./attributes/hx-prompt.md"),
    ("hx-replace-url", "./attributes/hx-replace-url.md"),
    ("hx-request", "./attributes/hx-request.md"),
    ("hx-sync", "./attributes/hx-sync.md"),
    ("hx-validate", "./attributes/hx-validate.md")
);

pub enum VariadicCompletion {
    HxSwap(&'static [HxCompletion; 8]),
    HxTarget(&'static [HxCompletion; 5]),
    HxBoost(&'static [HxCompletion; 2]),
    HxTrigger(&'static [HxCompletion; 14]),
    HxExt(&'static [HxCompletion; 22]),
    HxPushUrl(&'static [HxCompletion; 2]),
    HxSwapOob(&'static [HxCompletion; 9]),
    HxHistory(&'static [HxCompletion; 1]),
    HxParams(&'static [HxCompletion; 3]),
    HxReplaceUrl(&'static [HxCompletion; 2]),
    HxSync(&'static [HxCompletion; 4]),
}
pub static HX_ATTRIBUTE_VALUES: phf::Map<&'static str, VariadicCompletion> = phf::phf_map! {
    "hx-swap" => VariadicCompletion::HxSwap(
        build_completion!(("innerHTML", "./hx-swap/innerHTML.md"),
        ("outerHTML", "./hx-swap/outerHTML.md"),
        ("afterbegin", "./hx-swap/afterbegin.md"),
        ("afterend", "./hx-swap/afterend.md"),
        ("beforebegin", "./hx-swap/beforebegin.md"),
        ("beforeend", "./hx-swap/beforeend.md"),
        ("delete", "./hx-swap/delete.md"),
        ("none", "./hx-swap/none.md")
    )),

    "hx-target" => VariadicCompletion::HxTarget(build_completion![
        ("closest", "./hx-target/closest.md"),
        ("find", "./hx-target/find.md"),
        ("next", "./hx-target/next.md"),
        ("prev", "./hx-target/prev.md"),
        ("this", "./hx-target/this.md")
    ]),

    "hx-boost" => VariadicCompletion::HxBoost(build_completion![
        ("true", "./hx-boost/true.md"),
        ("false", "./hx-boost/false.md")
    ]),

    "hx-trigger" => VariadicCompletion::HxTrigger(build_completion![
        ("click", "./hx-trigger/click.md"),
        ("once", "./hx-trigger/once.md"),
        ("changed", "./hx-trigger/changed.md"),
        ("delay:", "./hx-trigger/delay.md"),
        ("throttle:", "./hx-trigger/throttle.md"),
        ("from:", "./hx-trigger/from.md"),
        ("target:", "./hx-trigger/target.md"),
        ("consume", "./hx-trigger/consume.md"),
        ("queue:", "./hx-trigger/queue.md"),
        ("keyup", "./hx-trigger/keyup.md"),
        ("load", "./hx-trigger/load.md"),
        ("revealed", "./hx-trigger/revealed.md"),
        ("intersect", "./hx-trigger/intersect.md"),
        ("every", "./hx-trigger/every.md")
    ]),

    "hx-ext" => VariadicCompletion::HxExt(build_completion![
        ("ajax-header", "./hx-ext/ajax-header.md"),
        ("alpine-morph", "./hx-ext/alpine-morph.md"),
        ("class-tools", "./hx-ext/class-tools.md"),
        ("client-side-templates", "./hx-ext/client-side-templates.md"),
        ("debug", "./hx-ext/debug.md"),
        ("disable-element", "./hx-ext/disable-element.md"),
        ("event-header", "./hx-ext/event-header.md"),
        ("head-support", "./hx-ext/head-support.md"),
        ("include-vals", "./hx-ext/include-vals.md"),
        ("json-enc", "./hx-ext/json-enc.md"),
        ("morph", "./hx-ext/morph.md"),
        ("loading-states", "./hx-ext/loading-states.md"),
        ("method-override", "./hx-ext/method-override.md"),
        ("morphdom-swap", "./hx-ext/morphdom-swap.md"),
        ("multi-swap", "./hx-ext/multi-swap.md"),
        ("path-deps", "./hx-ext/path-deps.md"),
        ("preload", "./hx-ext/preload.md"),
        ("remove-me", "./hx-ext/remove-me.md"),
        ("response-targets", "./hx-ext/response-targets.md"),
        ("restored", "./hx-ext/restored.md"),
        ("sse", "./hx-ext/sse.md"),
        ("ws", "./hx-ext/ws.md")
    ]),

    "hx-push-url" => VariadicCompletion::HxPushUrl(build_completion![
        ("true", "./hx-push-url/true.md"),
        ("false", "./hx-push-url/false.md")
    ]),

    "hx-swap-oob" => VariadicCompletion::HxSwapOob(build_completion![
        ("true", "./hx-swap-oob/true.md"),
        ("innerHTML", "./hx-swap/innerHTML.md"),
        ("outerHTML", "./hx-swap/outerHTML.md"),
        ("afterbegin", "./hx-swap/afterbegin.md"),
        ("afterend", "./hx-swap/afterend.md"),
        ("beforebegin", "./hx-swap/beforebegin.md"),
        ("beforeend", "./hx-swap/beforeend.md"),
        ("delete", "./hx-swap/delete.md"),
        ("none", "./hx-swap/none.md")
    ]),

    "hx-history" => VariadicCompletion::HxHistory(build_completion![
        ("false", "./hx-history/false.md")
    ]),

    "hx-params" => VariadicCompletion::HxParams(build_completion!(
        ("*", "./hx-params/star.md"),
        ("none", "./hx-params/none.md"),
        ("not", "./hx-params/not.md")
    )),

    "hx-replace-url" => VariadicCompletion::HxReplaceUrl(build_completion![
        ("true", "./hx-replace-url/true.md"),
        ("false", "./hx-replace-url/false.md")
    ]),

    "hx-sync" => VariadicCompletion::HxSync(build_completion![
        ("drop", "./hx-sync/drop.md"),
        ("abort", "./hx-sync/abort.md"),
        ("replace", "./hx-sync/replace.md"),
        ("queue", "./hx-sync/queue.md")
    ])
};
