//! Test host for file services: records requests and lets a test complete
//! them with chosen results.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::platform::{Done, FileHandle, Files, OpenResult, Opened, SaveResult, SaveTarget};

/// A save request as the fake received it.
pub struct SaveRequest {
    /// `Some(name)` for an overwrite of the file `name`, `None` for a request to ask.
    pub overwrite: Option<String>,
    /// The name suggested when asking.
    pub suggested: Option<String>,
    pub bytes: Vec<u8>,
    done: Option<Done<SaveResult>>,
}

#[derive(Default)]
struct State {
    can_overwrite: bool,
    opens: VecDeque<Done<OpenResult>>,
    saves: VecDeque<SaveRequest>,
    shown: Vec<(String, bool)>,
}

/// Shared handle: one clone goes into the `Host`, the test keeps another.
#[derive(Clone, Default)]
pub struct FakeFiles(Rc<RefCell<State>>);

impl FakeFiles {
    /// A host that can overwrite (`true`) or only download (`false`).
    pub fn new(can_overwrite: bool) -> Self {
        let fake = Self::default();
        fake.0.borrow_mut().can_overwrite = can_overwrite;
        fake
    }

    pub fn pending_opens(&self) -> usize {
        self.0.borrow().opens.len()
    }

    pub fn pending_saves(&self) -> usize {
        self.0.borrow().saves.len()
    }

    /// Target and bytes of the oldest pending save: (overwrite, suggested, bytes).
    pub fn save_request(&self) -> (Option<String>, Option<String>, Vec<u8>) {
        let s = self.0.borrow();
        let r = s.saves.front().expect("a pending save");
        (r.overwrite.clone(), r.suggested.clone(), r.bytes.clone())
    }

    /// Complete the oldest open request.
    pub fn complete_open(&self, result: OpenResult) {
        let done = self
            .0
            .borrow_mut()
            .opens
            .pop_front()
            .expect("a pending open");
        done(result);
    }

    /// Complete the oldest open request with the file `name` holding `bytes`.
    pub fn open_file(&self, name: &str, bytes: &[u8]) {
        self.complete_open(Some(Ok(Opened {
            handle: FileHandle::new(name, ()),
            bytes: bytes.to_vec(),
        })));
    }

    /// Complete the oldest save request; returns its bytes.
    pub fn complete_save(&self, result: SaveResult) -> Vec<u8> {
        let mut r = self
            .0
            .borrow_mut()
            .saves
            .pop_front()
            .expect("a pending save");
        (r.done.take().unwrap())(result);
        r.bytes
    }

    /// Complete the oldest save request successfully as the file `name`.
    pub fn save_as(&self, name: &str) -> Vec<u8> {
        self.complete_save(Some(Ok(FileHandle::new(name, ()))))
    }

    /// Every `show_document` call so far, as (title, unsaved).
    pub fn shown(&self) -> Vec<(String, bool)> {
        self.0.borrow().shown.clone()
    }
}

impl Files for FakeFiles {
    fn open(&self, done: Done<OpenResult>) {
        self.0.borrow_mut().opens.push_back(done);
    }

    fn save(&self, target: SaveTarget, bytes: Vec<u8>, done: Done<SaveResult>) {
        let (overwrite, suggested) = match target {
            SaveTarget::Overwrite(h) => (Some(h.name().to_owned()), None),
            SaveTarget::Ask { suggested_name } => (None, Some(suggested_name)),
        };
        self.0.borrow_mut().saves.push_back(SaveRequest {
            overwrite,
            suggested,
            bytes,
            done: Some(done),
        });
    }

    fn can_overwrite(&self) -> bool {
        self.0.borrow().can_overwrite
    }

    fn show_document(&self, title: &str, unsaved: bool) {
        self.0.borrow_mut().shown.push((title.to_owned(), unsaved));
    }
}
