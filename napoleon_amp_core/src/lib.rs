use std::sync::{Mutex, MutexGuard};

pub mod data;
pub mod instance;
mod paths;

pub fn unwrap_lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    mutex.lock().expect("Poisoned lock")
}
