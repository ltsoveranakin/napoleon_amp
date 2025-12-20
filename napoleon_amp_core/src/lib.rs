use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod data;
pub mod instance;
mod paths;

pub fn unlock_mutex<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    mutex.lock().expect("Poisoned lock")
}

pub fn write_rwlock<T>(rw_lock: &RwLock<T>) -> RwLockWriteGuard<T> {
    rw_lock.write().expect("Poisoned lock")
}

pub fn read_rwlock<T>(rw_lock: &RwLock<T>) -> RwLockReadGuard<T> {
    rw_lock.read().expect("Poisoned lock")
}
