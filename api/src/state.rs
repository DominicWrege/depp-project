use crate::api::{AssignmentId, IliasId};
use crate::deep_project;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
#[derive(Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}

impl State {
    pub fn new() -> State {
        State {
            inner: Arc::new(InnerState {
                pending_results: dashmap::DashMap::new(),
            }),
        }
    }
}

pub struct InnerState {
    pub pending_results: dashmap::DashMap<IliasId, deep_project::AssignmentResult>,
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
