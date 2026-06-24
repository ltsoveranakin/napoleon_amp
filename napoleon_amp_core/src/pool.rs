use crate::content::SaveData;
use serbytes::prelude::{SerBytes, SerBytesFs};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

type WeakArc<T> = std::sync::Weak<T>;

type GetPathFn<K> = Box<dyn Fn(&K) -> PathBuf + Send>;

struct DataPoolInner<K, V> {
    map: HashMap<K, WeakArc<V>>,
    get_path_fn: GetPathFn<K>,
}

impl<K, V> DataPoolInner<K, V>
where
    K: Hash + Eq,
    V: SerBytesFs + for<'a> SaveData<&'a K>,
{
    fn get_or_load_value_arc<F>(&mut self, input: K, or_insert: F) -> Arc<V>
    where
        F: FnOnce() -> V,
    {
        match self.map.entry(input) {
            Entry::Vacant(v) => {
                let (strong, weak) =
                    Self::get_strong_weak_pair(&self.get_path_fn, &v.key(), or_insert);

                v.insert(weak);

                strong
            }

            Entry::Occupied(mut o) => {
                if let Some(upgraded) = WeakArc::upgrade(o.get()) {
                    upgraded
                } else {
                    let (strong, weak) =
                        Self::get_strong_weak_pair(&self.get_path_fn, &o.key(), or_insert);

                    o.insert(weak);

                    strong
                }
            }
        }
    }

    // pub(crate) fn insert_if_empty<F>(&self, key: K, insert_fn: F)
    // where
    //     F: FnOnce() -> V,
    // {
    //     let path = (self.get_path_fn)(&key);
    // }

    // fn get_or_load_value_mut<LF, R>(&mut self, input: K, cb: LF) -> R
    // where
    //     LF: FnOnce(&mut V) -> R,
    // {
    //     let value_rwlock = self.get_or_load_value_arc(input);
    //
    //     cb(&mut write_rwlock(&value_rwlock))
    // }

    fn get_strong_weak_pair<F>(
        get_path_fn: &GetPathFn<K>,
        key: &K,
        or_insert: F,
    ) -> (Arc<V>, WeakArc<V>)
    where
        F: FnOnce() -> V,
    {
        let path = get_path_fn(key);

        let strong = Arc::new(V::from_file_path(path).unwrap_or_else(|_| {
            let value = or_insert();

            value.save_data(key).expect("Unable to save data");

            value
        }));

        let weak = Arc::downgrade(&strong);

        (strong, weak)
    }
}

pub struct DataPool<K, V> {
    inner: Mutex<DataPoolInner<K, V>>,
}

impl<K, V> DataPool<K, V>
where
    K: Hash + Eq,
    V: SerBytes + for<'a> SaveData<&'a K>,
{
    pub(crate) fn new<F>(get_path_fn: F) -> Self
    where
        F: Fn(&K) -> PathBuf + Send + 'static,
    {
        Self {
            inner: Mutex::new(DataPoolInner {
                map: HashMap::new(),
                get_path_fn: Box::new(get_path_fn),
            }),
        }
    }

    pub(crate) fn get_or_load_value_arc<F>(&self, key: K, or_insert: F) -> Arc<V>
    where
        F: FnOnce() -> V,
    {
        self.get_inner().get_or_load_value_arc(key, or_insert)
    }

    pub(crate) fn get_or_load_value_arc_default(&self, key: K) -> Arc<V>
    where
        V: Default,
    {
        self.get_or_load_value_arc(key, || V::default())
    }

    pub(crate) fn get_or_load_value<LF, R, F>(&self, key: K, cb: LF, or_insert: F) -> R
    where
        LF: FnOnce(&V) -> R,
        F: FnOnce() -> V,
    {
        let arc_value = self.get_or_load_value_arc(key, or_insert);

        cb(&arc_value)
    }

    fn get_inner(&self) -> MutexGuard<'_, DataPoolInner<K, V>> {
        self.inner.lock().unwrap()
    }
}
