use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod data;
pub mod instance;
mod paths;
mod net;

static POISONED_LOCK_MESSAGE: &str = "Poisoned lock";

pub fn unlock_mutex<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().expect(POISONED_LOCK_MESSAGE)
}

#[derive(Debug)]
pub struct WriteWrapper<'a, T>(RwLockWriteGuard<'a, T>);

#[derive(Debug)]
pub struct ReadWrapper<'a, T>(RwLockReadGuard<'a, T>);

impl<'a, T> WriteWrapper<'a, T> {
    pub(crate) fn new(value: RwLockWriteGuard<'a, T>) -> Self {
        // let bt = Backtrace::new();
        // println!("Create write {:?}", bt);

        Self(value)
    }
}

impl<'a, T> ReadWrapper<'a, T> {
    pub(crate) fn new(value: RwLockReadGuard<'a, T>) -> Self {
        // let bt = Backtrace::new();
        // println!("Create read {:?}", bt);

        Self(value)
    }
}

impl<'a, T> Drop for WriteWrapper<'a, T> {
    fn drop(&mut self) {
        // println!("Dropping write");
    }
}

impl<'a, T> Drop for ReadWrapper<'a, T> {
    fn drop(&mut self) {
        // println!("Dropping read");
    }
}

impl<'a, T> Deref for WriteWrapper<'a, T> {
    type Target = RwLockWriteGuard<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> Deref for ReadWrapper<'a, T> {
    type Target = RwLockReadGuard<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for WriteWrapper<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> DerefMut for ReadWrapper<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> From<RwLockReadGuard<'a, T>> for ReadWrapper<'a, T> {
    fn from(value: RwLockReadGuard<'a, T>) -> Self {
        Self::new(value)
    }
}

pub fn write_rwlock<T: Debug>(rw_lock: &RwLock<T>) -> WriteWrapper<T> {
    // println!("try write {:?}", rw_lock);
    // let bt = Backtrace::new();
    // println!("{:?}", bt);
    let l = rw_lock.write().expect(POISONED_LOCK_MESSAGE);
    // println!("obtained write");

    WriteWrapper::new(l)
}

pub fn read_rwlock<T>(rw_lock: &RwLock<T>) -> ReadWrapper<'_, T> {
    // println!("try read");
    let r = rw_lock.read().expect(POISONED_LOCK_MESSAGE);
    // println!("obtained read");

    ReadWrapper::new(r)
}
