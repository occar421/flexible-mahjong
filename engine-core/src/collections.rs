use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::iter::FromIterator;
use itertools::Itertools;

pub(crate) struct MultiBTreeSet<T> {
    map: BTreeMap<T, usize>
}

impl<T: Ord + Clone> MultiBTreeSet<T> {
    pub fn insert(&mut self, value: T) -> bool {
        if let Some(n) = self.map.get_mut(&value) {
            *n += 1;
        } else {
            self.map.insert(value, 1);
        }
        true
    }

    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
        where T: Borrow<Q>,
              Q: Ord
    {
        if let Some(n) = self.map.get_mut(value) {
            *n -= 1;
            if n == &0 {
                self.map.remove(value);
            }
            true
        } else {
            false
        }
    }

    pub fn get_by_buckets(&self) -> std::collections::btree_map::Iter<T, usize> {
        self.map.iter()
    }

    pub fn clone(&self) -> Self {
        MultiBTreeSet { map: self.map.clone() }
    }

    pub fn len(&self) -> usize {
        self.map.iter().fold(0, |acc, (_, &n)| acc + n)
    }
}

impl<T: Ord + Copy> FromIterator<T> for MultiBTreeSet<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> MultiBTreeSet<T> {
        MultiBTreeSet {
            map: BTreeMap::from_iter(
                iter.into_iter()
                    .group_by(|e| e.clone())
                    .into_iter()
                    .map(|(k, gv)| (k, gv.count()))
            )
        }
    }
}
