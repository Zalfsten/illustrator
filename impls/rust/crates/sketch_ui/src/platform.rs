//! The host boundary (system-architecture "Host services pass through a
//! single host boundary"): every host service the App uses comes from the
//! [`Platform`] supplied when it starts. Optional capabilities are `Option`s.

use label_layout::Typesetter;

/// Host services of the App.
pub trait Platform {
    /// Optional capability: label typesetting (`None` on a host without one).
    fn typesetter(&self) -> Option<&dyn Typesetter>;
}

/// A host as a set of boxed services; `main.rs` builds one per target.
pub struct Host {
    typesetter: Option<Box<dyn Typesetter>>,
}

impl Host {
    pub fn new(typesetter: Option<Box<dyn Typesetter>>) -> Self {
        Self { typesetter }
    }
}

impl Platform for Host {
    fn typesetter(&self) -> Option<&dyn Typesetter> {
        self.typesetter.as_deref()
    }
}
