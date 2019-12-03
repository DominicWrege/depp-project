use crate::api;
use crate::api::IliasId;
use crate::config::{Assignment, AssignmentId};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}

impl State {
    pub fn new(c: HashMap<AssignmentId, Assignment>) -> State {
        State {
            inner: Arc::new(InnerState {
                pending_results: Default::default(),
                config: c,
            }),
        }
    }
}

#[derive(Debug)]
pub struct InnerState {
    pub pending_results: RwLock<HashMap<IliasId, api::AssignmentResult>>,
    pub config: HashMap<AssignmentId, Assignment>,
}

impl Deref for State {
    type Target = Arc<InnerState>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
