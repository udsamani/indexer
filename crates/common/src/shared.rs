use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct SharedRef<T> {
    inner: Arc<Mutex<T>>,
}

impl<T: Default> Default for SharedRef<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Clone for SharedRef<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> SharedRef<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock().unwrap()
    }
}

pub struct SharedAsyncRef<T> {
    inner: Arc<tokio::sync::RwLock<T>>,
}

impl<T: Default> Default for SharedAsyncRef<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Clone for SharedAsyncRef<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> SharedAsyncRef<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(tokio::sync::RwLock::new(value)),
        }
    }

    pub async fn lock(&self) -> tokio::sync::RwLockWriteGuard<T> {
        self.inner.write().await
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<T> {
        self.inner.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<T> {
        self.inner.write().await
    }
}

pub struct SharedRwRef<T> {
    inner: Arc<RwLock<T>>,
}

impl<T: Default> Default for SharedRwRef<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Clone for SharedRwRef<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> SharedRwRef<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.inner.write().unwrap()
    }
}
