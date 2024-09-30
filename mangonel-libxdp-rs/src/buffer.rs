pub trait Buffer<T> {
    /// Return the number of items in the buffer.
    fn count(&self) -> u32;

    /// Return the number of items that can be pushed before [`Buffer::push`]
    /// starts returning a value.
    fn free(&self) -> u32;

    /// Return [None] if the buffer is empty.
    fn pop(&mut self) -> Option<T>;

    /// Return a previous value if the buffer is full.
    fn push(&mut self, value: T) -> Option<T>;
}

impl<T> Buffer<T> for std::collections::VecDeque<T> {
    #[inline(always)]
    fn count(&self) -> u32 {
        self.len() as u32
    }

    #[inline(always)]
    fn free(&self) -> u32 {
        (self.capacity() - self.len()) as u32
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
        self.pop_front()
    }

    #[inline(always)]
    fn push(&mut self, value: T) -> Option<T> {
        if self.free() == 0 {
            let overwritten = self.pop_front();
            self.push_back(value);

            return overwritten;
        }

        self.push_back(value);

        return None;
    }
}
