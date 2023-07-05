#[derive(Debug, Clone)]
pub struct Aggregate<K, V> {
    m: std::collections::HashMap<K, V>,
    sum: V,
}

impl<K, V> Default for Aggregate<K, V>
where
    V: Default,
{
    fn default() -> Self {
        Self {
            m: Default::default(),
            sum: Default::default(),
        }
    }
}

impl<K, V> PartialEq for Aggregate<K, V>
where
    K: Eq + std::hash::Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.m == other.m && self.sum == other.sum
    }
}

impl<K, V> Eq for Aggregate<K, V>
where
    K: Eq + std::hash::Hash,
    V: Eq,
{
}

impl<K, V> Aggregate<K, V> {
    pub fn sum(&self) -> V
    where
        V: Copy,
    {
        self.sum
    }

    pub fn is_empty(&self) -> bool {
        self.m.is_empty()
    }

    pub fn len(&self) -> usize {
        self.m.len()
    }

    pub fn add(&mut self, key: K, value: V)
    where
        K: Copy + Eq + std::hash::Hash,
        V: Copy + Default + std::ops::AddAssign,
    {
        *(self.m.entry(key).or_default()) += value;
        self.sum += value;
    }

    pub fn get(&self, key: K) -> Option<V>
    where
        K: Copy + Eq + std::hash::Hash,
        V: Copy,
    {
        self.m.get(&key).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, V)> + '_
    where
        K: Copy,
        V: Copy,
    {
        self.m.iter().map(|(&k, &v)| (k, v))
    }
}

impl<K, V> FromIterator<(K, V)> for Aggregate<K, V>
where
    K: Copy + Eq + std::hash::Hash,
    V: Copy + Default + std::ops::AddAssign,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut agg = Aggregate::<K, V>::default();
        for (k, v) in iter {
            agg.add(k, v);
        }
        agg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate() {
        let mut agg = Aggregate::<&'static str, i32>::default();
        assert!(agg.is_empty());
        assert_eq!(agg.sum(), 0);

        agg.add("a", 10);
        agg.add("b", -100);
        assert!(!agg.is_empty());
        assert_eq!(agg.get("a").unwrap(), 10);
        assert_eq!(agg.get("b").unwrap(), -100);
        assert!(agg.get("c").is_none());
        assert_eq!(agg.sum(), -90);

        agg.add("a", -3);
        agg.add("c", 0);
        assert_eq!(agg.get("a").unwrap(), 7);
        assert_eq!(agg.get("b").unwrap(), -100);
        assert_eq!(agg.get("c").unwrap(), 0);
        assert_eq!(agg.sum(), -93);

        let mut vec = agg.iter().collect::<Vec<_>>();
        vec.sort();
        assert_eq!(vec, vec![("a", 7), ("b", -100), ("c", 0)]);

        let agg2 = vec.into_iter().collect::<Aggregate<_, _>>();
        assert_eq!(agg, agg2);
    }
}
