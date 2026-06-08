#![deny(unused_must_use)]

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, SystemTime};

pub mod assets;
pub mod content;
pub mod discord_rpc;
pub mod instance;
mod net;
pub mod paths;
mod pool;
mod resetable_once_cell;

pub use simple_id;

static POISONED_LOCK_MESSAGE: &str = "Poisoned lock";

pub type ReadGuard<'a, T> = RwLockReadGuard<'a, T>;
pub type WriteGuard<'a, T> = RwLockWriteGuard<'a, T>;

pub fn unlock_mutex<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().expect(POISONED_LOCK_MESSAGE)
}

pub fn write_rwlock<T>(rw_lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    rw_lock.write().expect(POISONED_LOCK_MESSAGE)
}

pub fn read_rwlock<T>(rw_lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    rw_lock.read().expect(POISONED_LOCK_MESSAGE)
}

pub(crate) fn time_now() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System clock went backwards")
}
