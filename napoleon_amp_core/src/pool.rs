use crate::{read_rwlock, write_rwlock};
use serbytes::prelude::{SerBytes, SerBytesFs};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

type WeakArc<T> = std::sync::Weak<T>;

struct DataPoolInner<K, V, F> {
    map: HashMap<K, WeakArc<RwLock<V>>>,
    get_path_fn: F,
}

impl<K, V, F> DataPoolInner<K, V, F>
where
    K: Hash + Eq,
    V: SerBytes,
    F: Fn(&K) -> PathBuf,
{
    fn get_or_load_value_rwlock(&mut self, input: K) -> Arc<RwLock<V>> {
        match self.map.entry(input) {
            Entry::Vacant(v) => {
                let (strong, weak) = Self::get_strong_weak_pair(&self.get_path_fn, &v.key());

                v.insert(weak);

                strong
            }

            Entry::Occupied(mut o) => {
                if let Some(upgraded) = WeakArc::upgrade(o.get()) {
                    upgraded
                } else {
                    let (strong, weak) = Self::get_strong_weak_pair(&self.get_path_fn, &o.key());

                    o.insert(weak);

                    strong
                }
            }
        }
    }

    fn get_or_load_value<LF, R>(&mut self, input: K, cb: LF) -> R
    where
        LF: FnOnce(&V) -> R,
    {
        let value_rwlock = self.get_or_load_value_rwlock(input);

        cb(&read_rwlock(&value_rwlock))
    }

    fn get_or_load_value_mut<LF, R>(&mut self, input: K, cb: LF) -> R
    where
        LF: FnOnce(&mut V) -> R,
    {
        let value_rwlock = self.get_or_load_value_rwlock(input);

        cb(&mut write_rwlock(&value_rwlock))
    }

    fn get_strong_weak_pair(get_path_fn: &F, key: &K) -> (Arc<RwLock<V>>, WeakArc<RwLock<V>>) {
        let path = get_path_fn(key);

        let value = V::from_file_path(path).expect("Unable to read data from path into pool");

        let strong = Arc::new(RwLock::new(value));
        let weak = Arc::downgrade(&strong);

        (strong, weak)
    }
}

pub struct DataPool<K, V, F> {
    inner: Mutex<DataPoolInner<K, V, F>>,
}

impl<K, V, F> DataPool<K, V, F> {
    fn get_inner(&self) -> MutexGuard<'_, DataPoolInner<K, V, F>> {
        self.inner.lock().unwrap()
    }
}
