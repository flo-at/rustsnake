pub struct CyclicBuffer<T: Default + Clone> {
    segments: Vec<T>,
    head_index: usize,
    beyond_tail_index: usize,
    empty: bool,
}

impl<T: Default + Clone> CyclicBuffer<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            segments: vec![T::default(); max_size],
            head_index: 0,
            beyond_tail_index: 0,
            empty: true,
        }
    }

    pub fn count(&self) -> usize {
        if self.empty {
            0
        } else if self.beyond_tail_index > self.head_index {
            self.beyond_tail_index - self.head_index
        } else {
            self.beyond_tail_index + self.capacity() - self.head_index
        }
    }

    pub fn empty(&self) -> bool {
        self.empty
    }

    pub fn full(&self) -> bool {
        self.count() == self.capacity()
    }

    pub fn capacity(&self) -> usize {
        self.segments.len()
    }

    fn increment(&self, index: usize) -> usize {
        (index + 1) % self.capacity()
    }

    pub fn push(&mut self, value: T) -> bool {
        if self.beyond_tail_index == self.head_index && !self.empty {
            return false;
        }
        let new_tail_index = self.increment(self.beyond_tail_index);
        self.segments[self.beyond_tail_index] = value;
        self.beyond_tail_index = new_tail_index;
        self.empty = false;
        true
    }

    pub fn force_push(&mut self, value: T) {
        if self.beyond_tail_index == self.head_index && !self.empty {
            self.head_index = self.increment(self.head_index);
        }
        let new_tail_index = self.increment(self.beyond_tail_index);
        self.segments[self.beyond_tail_index] = value;
        self.beyond_tail_index = new_tail_index;
        self.empty = false;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.empty {
            return None;
        }
        let head_index = self.head_index;
        self.head_index = self.increment(self.head_index);
        self.empty = self.head_index == self.beyond_tail_index;
        Some(std::mem::take(&mut self.segments[head_index]))
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            buffer: self,
            position: 0,
        }
    }
}

#[derive(Clone)]
pub struct Iter<'a, T: Default + Clone> {
    buffer: &'a CyclicBuffer<T>,
    position: usize,
}

impl<'a, T: Default + Clone> core::iter::Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position == self.buffer.count() {
            return None;
        }
        let cyclic_position = (self.buffer.head_index + self.position) % self.buffer.capacity();
        let segment = &self.buffer.segments[cyclic_position];
        self.position += 1;
        Some(segment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity() {
        let buf = CyclicBuffer::<u32>::new(3);
        assert_eq!(buf.capacity(), 3);
    }

    #[test]
    fn empty() {
        let mut buf = CyclicBuffer::<u32>::new(3);
        assert!(buf.empty());
        assert_eq!(buf.count(), 0);
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn one_element() {
        let mut buf = CyclicBuffer::<u32>::new(3);
        assert!(buf.push(1));
        assert!(!buf.empty());
        assert_eq!(buf.count(), 1);
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn full() {
        let mut buf = CyclicBuffer::<u32>::new(3);
        assert!(buf.push(1));
        assert!(buf.push(2));
        assert!(buf.push(3));
        assert!(buf.full());
        assert!(!buf.push(4));
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(3));
    }

    #[test]
    fn force_push() {
        let mut buf = CyclicBuffer::<u32>::new(3);
        assert!(buf.push(1));
        assert!(buf.push(2));
        assert!(buf.push(3));
        assert!(buf.full());
        buf.force_push(4);
        assert!(buf.full());
        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
    }

    #[test]
    fn iter() {
        let mut buf = CyclicBuffer::<u32>::new(3);
        assert!(buf.push(1));
        assert!(buf.push(2));
        assert!(buf.push(3));
        assert!(buf.full());
        let mut iter = buf.iter();
        assert_eq!(*iter.next().unwrap_or(&0), 1);
        assert_eq!(*iter.next().unwrap_or(&0), 2);
        assert_eq!(*iter.next().unwrap_or(&0), 3);
        assert!(iter.next().is_none());
    }
}
