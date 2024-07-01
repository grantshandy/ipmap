#![allow(dead_code)]

use std::{
    hash::{BuildHasherDefault, Hash},
    time::Duration,
};

use time::OffsetDateTime;

type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>;

/// A concurrent, auto-expiring set where elements only exist for a given amount of time before "expiring".
pub struct ExpirySet<V> {
    expire: Duration,
    contents: FxDashMap<V, OffsetDateTime>,
}

impl<V: Hash + Eq> ExpirySet<V> {
    /// Create a new [ExpirySet] with a given [Duration].
    pub fn new(expire: Duration) -> Self {
        Self {
            expire,
            contents: FxDashMap::default(),
        }
    }

    /// Insert an element into the set if it doesn't already exist.
    ///
    /// Returns true if the element did not already exist or previously expired.
    pub fn insert(&self, elem: V) -> bool {
        let now = OffsetDateTime::now_utc();

        match self.contents.insert(elem, now) {
            Some(prev) => (now - prev) >= self.expire,
            None => true,
        }
    }

    /// Returns true if the value exists in the set and isn't expired.
    pub fn contains(&self, elem: &V) -> bool {
        let now = OffsetDateTime::now_utc();

        self.contents
            .remove_if(elem, |_, prev| ((now - *prev) >= self.expire))
            .is_none()
    }

    /// Cleans all expired elements from the set (optional, cleans up memory)
    pub fn clean(&self) {
        let now = OffsetDateTime::now_utc();

        self.contents
            .iter()
            .filter(|kv| (now - *kv.value()) >= self.expire)
            .for_each(|kv| {
                self.contents.remove(kv.key());
            });

        todo!()
    }

    /// The number of elements in the set
    pub fn size(&self) -> usize {
        self.clean();

        self.contents.len()
    }
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use super::ExpirySet;

    const EXPIRE_DURATION: Duration = Duration::from_millis(25);

    #[test]
    fn insert() {
        let set: ExpirySet<()> = ExpirySet::new(EXPIRE_DURATION);

        // inserting changes set (true)
        assert!(set.insert(()));

        // inserting does not change set (false)
        assert!(!set.insert(()));

        thread::sleep(EXPIRE_DURATION);

        // inserting after expire duration changes set (true)
        assert!(set.insert(()));
    }

    #[test]
    fn contains() {
        let set: ExpirySet<()> = ExpirySet::new(EXPIRE_DURATION);

        assert!(set.insert(()));

        // set contains after insertion (true)
        assert!(set.contains(&()));
        thread::sleep(EXPIRE_DURATION);

        // set does not contain after expiry (false)
        assert!(!set.contains(&()));
    }
}
