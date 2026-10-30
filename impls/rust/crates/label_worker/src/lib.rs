//! Web Worker that typesets label lines (#60): the browser's typesetter.
//!
//! Messages are JSON strings ([`label_layout::wire`]). At start the worker
//! typesets one line to load Typst's fonts and library, then posts `ready`;
//! afterwards it answers each [`WireRequest`] with one `line` message.

use constraint_runtime::Markup;
use label_layout::wire::{WireRequest, WorkerMessage};
use label_layout::LineTypesetter;
use label_typst::TypstTypesetter;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

#[wasm_bindgen(start)]
pub fn start() {
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    let typesetter = TypstTypesetter::new();
    // Warm-up: the first line loads the font book and library (cold start).
    let _ = typesetter.typeset_line("x", Markup::Text, 10.0, "");

    let reply = scope.clone();
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        let Some(text) = e.data().as_string() else {
            return;
        };
        // A request the app cannot have sent is dropped; the app's request
        // then stays pending, which it shows as such.
        let Ok(req) = WireRequest::from_json(&text) else {
            return;
        };
        let r = req.request();
        let result = typesetter.typeset_line(&r.line, r.markup, r.size_pt(), &r.preamble);
        post(&reply, &WorkerMessage::line(req.id, &result));
    });
    scope.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    // The worker lives as long as the page; so does its handler.
    on_message.forget();
    post(&scope, &WorkerMessage::Ready {});
}

fn post(scope: &DedicatedWorkerGlobalScope, msg: &WorkerMessage) {
    // Typst output is finite, so writing cannot fail.
    if let Ok(json) = msg.to_json() {
        let _ = scope.post_message(&JsValue::from_str(&json));
    }
}
