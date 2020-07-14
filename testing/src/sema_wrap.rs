//! Semaphore wrapper for limiting concurrent tests.
use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};

#[derive(Clone, Debug)]
pub struct SemWrap<T>(Arc<SemWrapInner<T>>);
#[derive(Debug)]
pub struct SemWrapInner<T> {
    sem: Semaphore,
    a: T,
}

impl<T> SemWrap<T> {
    pub async fn acquire(&self) -> SemRef<'_, T> {
        let guard = self.0.sem.acquire().await;
        SemRef {
            _guard: guard,
            a: &self.0.a,
        }
    }
    pub fn new(a: T, max_sema: usize) -> Self {
        Self(Arc::new(SemWrapInner {
            a,
            sem: Semaphore::new(max_sema),
        }))
    }
}

pub struct SemRef<'a, T> {
    a: &'a T,
    _guard: SemaphorePermit<'a>,
}

impl<'a, T> std::ops::Deref for SemRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.a
    }
}
