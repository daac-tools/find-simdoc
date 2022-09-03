pub struct ShingleIter<'a, T> {
    tokens: &'a [T],
    window_size: usize,
    position: usize,
}

impl<'a, T> ShingleIter<'a, T> {
    pub fn new(tokens: &'a [T], window_size: usize) -> Self {
        Self {
            tokens,
            window_size,
            position: 0,
        }
    }
}

impl<'a, T> Iterator for ShingleIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.tokens.len() < self.position + self.window_size {
            return None;
        }
        let window = &self.tokens[self.position..self.position + self.window_size];
        self.position += 1;
        Some(window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q1() {
        let tokens = vec!["a", "b", "c"];
        let mut iter = ShingleIter::new(&tokens, 1);
        assert_eq!(iter.next(), Some(&tokens[0..1]));
        assert_eq!(iter.next(), Some(&tokens[1..2]));
        assert_eq!(iter.next(), Some(&tokens[2..3]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_q2() {
        let tokens = vec!["a", "b", "c"];
        let mut iter = ShingleIter::new(&tokens, 2);
        assert_eq!(iter.next(), Some(&tokens[0..2]));
        assert_eq!(iter.next(), Some(&tokens[1..3]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_q3() {
        let tokens = vec!["a", "b", "c"];
        let mut iter = ShingleIter::new(&tokens, 3);
        assert_eq!(iter.next(), Some(&tokens[0..3]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_q4() {
        let tokens = vec!["a", "b", "c"];
        let mut iter = ShingleIter::new(&tokens, 4);
        assert_eq!(iter.next(), None);
    }
}
