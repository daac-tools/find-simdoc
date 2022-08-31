#[derive(Clone, Copy)]
pub struct Bitset64(u64);

impl Bitset64 {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn add(mut self, i: usize) -> Self {
        assert!(i < 64);
        self.0 |= 1 << i;
        self
    }

    pub fn max(&self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            Some(63 - self.0.leading_zeros() as usize)
        }
    }

    pub fn inverse(mut self) -> Self {
        self.0 = !self.0;
        self
    }

    pub fn iter(&self) -> Bitset64Iter {
        Bitset64Iter(self.0)
    }

    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct Bitset64Iter(u64);

impl Iterator for Bitset64Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let numtz = self.0.trailing_zeros() as usize;
        let mask = 1 << numtz;
        self.0 ^= mask;
        Some(numtz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        // {}
        let mut s = Bitset64::new();
        assert_eq!(s.len(), 0);
        assert_eq!(s.is_empty(), true);
        assert_eq!(s.max(), None);
        assert_eq!(s.iter().collect::<Vec<_>>(), vec![]);

        // {2}
        s = s.add(2);
        assert_eq!(s.len(), 1);
        assert_eq!(s.is_empty(), false);
        assert_eq!(s.max(), Some(2));
        assert_eq!(s.iter().collect::<Vec<_>>(), vec![2]);

        // {2,9}
        s = s.add(9);
        assert_eq!(s.len(), 2);
        assert_eq!(s.is_empty(), false);
        assert_eq!(s.max(), Some(9));
        assert_eq!(s.iter().collect::<Vec<_>>(), vec![2, 9]);

        // {2,5,9}
        s = s.add(5);
        assert_eq!(s.len(), 3);
        assert_eq!(s.is_empty(), false);
        assert_eq!(s.max(), Some(9));
        assert_eq!(s.iter().collect::<Vec<_>>(), vec![2, 5, 9]);

        // {2,5,9}
        s = s.add(9);
        assert_eq!(s.len(), 3);
        assert_eq!(s.is_empty(), false);
        assert_eq!(s.max(), Some(9));
        assert_eq!(s.iter().collect::<Vec<_>>(), vec![2, 5, 9]);

        // !{2,5,9}
        s = s.inverse();
        assert_eq!(s.len(), 61);
        assert_eq!(s.is_empty(), false);
        assert_eq!(s.max(), Some(63));

        let mut expexted = vec![0, 1, 3, 4, 6, 7, 8];
        expexted.extend(10..64);
        assert_eq!(s.iter().collect::<Vec<_>>(), expexted);
    }
}
