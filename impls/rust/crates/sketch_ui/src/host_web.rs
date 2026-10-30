//! File services of the browser host: a file input for Open, a download for
//! Save (the page cannot write to a chosen file), the page title, and the
//! browser's leave warning while the drawing is unsaved.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{BeforeUnloadEvent, Blob, BlobPropertyBag, HtmlAnchorElement, HtmlInputElement, Url};

use crate::platform::{Done, FileHandle, Files, OpenResult, Opened, SaveResult, SaveTarget};

/// Page title after the drawing's title.
const APP_TITLE: &str = "Constraint Sketch";

pub struct WebFiles {
    /// Read by the `beforeunload` listener.
    unsaved: Rc<Cell<bool>>,
}

impl WebFiles {
    pub fn new() -> Self {
        let unsaved = Rc::new(Cell::new(false));
        let flag = unsaved.clone();
        let warn = Closure::<dyn FnMut(BeforeUnloadEvent)>::new(move |e: BeforeUnloadEvent| {
            if flag.get() {
                e.prevent_default();
                e.set_return_value("unsaved changes");
            }
        });
        if let Some(w) = web_sys::window() {
            let _ =
                w.add_event_listener_with_callback("beforeunload", warn.as_ref().unchecked_ref());
        }
        warn.forget(); // lives as long as the page
        Self { unsaved }
    }
}

fn document() -> Result<web_sys::Document, String> {
    web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| "no document".to_owned())
}

fn js_error(e: JsValue) -> String {
    e.as_string()
        .or_else(|| e.dyn_ref::<js_sys::Error>().map(|e| e.message().into()))
        .unwrap_or_else(|| format!("{e:?}"))
}

/// The completion of an open request, taken by whichever event comes first.
type OpenDone = Rc<RefCell<Option<Done<OpenResult>>>>;

/// A hidden `<input type=file>`; `done` gets the chosen file, or `None` on cancel.
fn start_open(done: OpenDone) -> Result<(), String> {
    let doc = document()?;
    let input: HtmlInputElement = doc
        .create_element("input")
        .map_err(js_error)?
        .dyn_into()
        .map_err(|_| "not an input element".to_owned())?;
    input.set_type("file");
    input.set_accept(".json,application/json");
    input.set_hidden(true);
    let body = doc.body().ok_or("no body")?;
    body.append_child(&input).map_err(js_error)?;

    // Exactly one of `change` and `cancel` completes the request.
    let finish = {
        let input = input.clone();
        move |result: OpenResult| {
            input.remove();
            if let Some(done) = done.borrow_mut().take() {
                done(result);
            }
        }
    };
    let on_cancel = {
        let finish = finish.clone();
        Closure::once_into_js(move || finish(None))
    };
    let on_change = {
        let input = input.clone();
        Closure::once_into_js(move || {
            let Some(file) = input.files().and_then(|f| f.get(0)) else {
                return finish(None);
            };
            wasm_bindgen_futures::spawn_local(async move {
                let name = file.name();
                let read = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await;
                finish(Some(match read {
                    Ok(buf) => Ok(Opened {
                        handle: FileHandle::new(name, ()),
                        bytes: js_sys::Uint8Array::new(&buf).to_vec(),
                    }),
                    Err(e) => Err(js_error(e)),
                }));
            });
        })
    };
    input
        .add_event_listener_with_callback("cancel", on_cancel.unchecked_ref())
        .map_err(js_error)?;
    input
        .add_event_listener_with_callback("change", on_change.unchecked_ref())
        .map_err(js_error)?;
    input.click();
    Ok(())
}

/// Hand `bytes` to the user as a download named `name`.
fn download(name: &str, bytes: &[u8]) -> Result<(), String> {
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    let options = BlobPropertyBag::new();
    options.set_type("application/json");
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(js_error)?;
    let url = Url::create_object_url_with_blob(&blob).map_err(js_error)?;
    let a: HtmlAnchorElement = document()?
        .create_element("a")
        .map_err(js_error)?
        .dyn_into()
        .map_err(|_| "not an anchor element".to_owned())?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    let _ = Url::revoke_object_url(&url);
    Ok(())
}

impl Files for WebFiles {
    fn open(&self, done: Done<OpenResult>) {
        let done = Rc::new(RefCell::new(Some(done)));
        if let Err(e) = start_open(done.clone()) {
            if let Some(done) = done.borrow_mut().take() {
                done(Some(Err(e)));
            }
        }
    }

    fn save(&self, target: SaveTarget, bytes: Vec<u8>, done: Done<SaveResult>) {
        let name = match target {
            SaveTarget::Ask { suggested_name } => suggested_name,
            // Never asked for (`can_overwrite` is false); download under its name.
            SaveTarget::Overwrite(h) => h.name().to_owned(),
        };
        done(Some(
            download(&name, &bytes).map(|()| FileHandle::new(name, ())),
        ));
    }

    fn can_overwrite(&self) -> bool {
        false
    }

    fn show_document(&self, title: &str, unsaved: bool) {
        self.unsaved.set(unsaved);
        if let Ok(doc) = document() {
            doc.set_title(&format!("{title} — {APP_TITLE}"));
        }
    }
}
