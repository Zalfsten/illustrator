//! Bookkeeping of a worker client, independent of the browser.

use std::collections::HashMap;

use label_layout::wire::{WireRequest, WorkerMessage};
use label_layout::{LineRequest, LineResult, TypesetterState};

/// Requests in flight, the queue before `ready`, and completed results.
pub struct Core {
    next_id: u32,
    state: TypesetterState,
    in_flight: HashMap<u32, LineRequest>,
    /// Requests made before the worker was ready, in order.
    queued: Vec<WireRequest>,
    completed: Vec<(LineRequest, LineResult)>,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Core {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            state: TypesetterState::Loading,
            in_flight: HashMap::new(),
            queued: Vec::new(),
            completed: Vec::new(),
        }
    }

    pub fn state(&self) -> TypesetterState {
        self.state.clone()
    }

    /// The message to post for `req` now, or `None` (queued until ready, a
    /// duplicate of a request in flight, or the worker failed).
    pub fn request(&mut self, req: LineRequest) -> Option<WireRequest> {
        if matches!(self.state, TypesetterState::Failed(_))
            || self.in_flight.values().any(|r| *r == req)
        {
            return None;
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let wire = WireRequest::new(id, &req);
        self.in_flight.insert(id, req);
        match self.state {
            TypesetterState::Ready => Some(wire),
            _ => {
                self.queued.push(wire);
                None
            }
        }
    }

    /// Handle a message from the worker; returns the queued requests to post
    /// once it is ready. Messages it cannot read are ignored.
    pub fn on_message(&mut self, text: &str) -> Vec<WireRequest> {
        match WorkerMessage::from_json(text) {
            Ok(WorkerMessage::Ready {}) => {
                if matches!(self.state, TypesetterState::Failed(_)) {
                    return Vec::new();
                }
                self.state = TypesetterState::Ready;
                std::mem::take(&mut self.queued)
            }
            Ok(WorkerMessage::Line { id, result }) => {
                if let Some(req) = self.in_flight.remove(&id) {
                    self.completed.push((req, result.into()));
                }
                Vec::new()
            }
            Err(_) => Vec::new(),
        }
    }

    /// The worker failed (to load or to run): no more answers.
    pub fn on_error(&mut self, message: &str) {
        self.state = TypesetterState::Failed(message.to_owned());
        self.queued.clear();
        self.in_flight.clear();
    }

    pub fn take_completed(&mut self) -> Vec<(LineRequest, LineResult)> {
        std::mem::take(&mut self.completed)
    }
}

#[cfg(test)]
#[path = "tests_core.rs"]
mod tests_core;
