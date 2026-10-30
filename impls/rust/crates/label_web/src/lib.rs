//! The browser's typesetter: a [`Typesetter`] that sends lines to the
//! `label_worker` Web Worker and receives them back (label-typesetting
//! "Typesetting never blocks interaction").
//!
//! - [`core`]: ids, the queue before the worker is ready, answer matching and
//!   state; pure Rust, tested natively.
//! - [`WorkerTypesetter`]: starts the worker on the first request and calls
//!   `wake` whenever a message arrives, so the app repaints.

pub mod core;

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use label_layout::wire::WireRequest;
use label_layout::{LineRequest, LineResult, Typesetter, TypesetterState};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, Event, MessageEvent, Worker};

use crate::core::Core;

/// Client of the typesetting worker at `url` (its loader script).
pub struct WorkerTypesetter {
    url: String,
    wake: Rc<dyn Fn()>,
    core: Rc<RefCell<Core>>,
    worker: OnceCell<Worker>,
}

impl WorkerTypesetter {
    /// `wake` is called after every message from the worker (e.g. to request
    /// a repaint). Nothing is loaded until the first request.
    pub fn new(url: &str, wake: impl Fn() + 'static) -> Self {
        Self {
            url: url.to_owned(),
            wake: Rc::new(wake),
            core: Rc::new(RefCell::new(Core::new())),
            worker: OnceCell::new(),
        }
    }

    /// The worker, started on first use; `None` if it could not be created.
    fn worker(&self) -> Option<&Worker> {
        if self.worker.get().is_none() {
            match Worker::new(&self.url) {
                Ok(w) => {
                    self.listen(&w);
                    let _ = self.worker.set(w);
                }
                Err(e) => {
                    let msg = e.as_string().unwrap_or_else(|| format!("{e:?}"));
                    self.core.borrow_mut().on_error(&msg);
                    return None;
                }
            }
        }
        self.worker.get()
    }

    fn listen(&self, worker: &Worker) {
        let (core, wake, w) = (self.core.clone(), self.wake.clone(), worker.clone());
        let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            if let Some(text) = e.data().as_string() {
                let flush = core.borrow_mut().on_message(&text);
                for req in flush {
                    post(&w, &req);
                }
            }
            wake();
        });
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        let (core, wake) = (self.core.clone(), self.wake.clone());
        // A script that fails to load fires a plain `Event`; an uncaught
        // error inside the worker (e.g. from the boot script) an `ErrorEvent`.
        let on_error = Closure::<dyn FnMut(Event)>::new(move |e: Event| {
            let msg = e
                .dyn_ref::<ErrorEvent>()
                .map(ErrorEvent::message)
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| "worker script failed to load".to_owned());
            core.borrow_mut().on_error(&msg);
            wake();
        });
        worker.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();
    }
}

fn post(worker: &Worker, req: &WireRequest) {
    if let Ok(json) = req.to_json() {
        let _ = worker.post_message(&JsValue::from_str(&json));
    }
}

impl Typesetter for WorkerTypesetter {
    fn request(&self, req: LineRequest) {
        let Some(worker) = self.worker() else {
            return;
        };
        let send = self.core.borrow_mut().request(req);
        if let Some(msg) = send {
            post(worker, &msg);
        }
    }

    fn take_completed(&self) -> Vec<(LineRequest, LineResult)> {
        self.core.borrow_mut().take_completed()
    }

    fn state(&self) -> TypesetterState {
        self.core.borrow().state()
    }
}
