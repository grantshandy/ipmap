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

        if let Some(mut prev) = self.contents.get_mut(&elem) {
            let expired = (now - *prev.value()) >= self.expire;

            if expired {
                *prev.value_mut() = now;
            }

            expired // previously expired
        } else {
            self.contents.insert(elem, now);
            true // element did not already exist
        }
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
}
