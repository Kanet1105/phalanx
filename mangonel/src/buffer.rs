use std::sync::Arc;

use crossbeam::queue::ArrayQueue;

use crate::descriptor::Descriptor;

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

impl Buffer<Descriptor> for std::collections::VecDeque<Descriptor> {
    fn count(&self) -> u32 {
        self.len() as u32
    }

    fn free(&self) -> u32 {
        (self.capacity() - self.len()) as u32
    }

    fn pop(&mut self) -> Option<Descriptor> {
        self.pop_front()
    }

    fn push(&mut self, value: Descriptor) -> Option<Descriptor> {
        if self.free() == 0 {
            let overflow = self.pop_front();
            self.push_back(value);

            return overflow;
        } else {
            self.push_back(value);

            return None;
        }
    }
}

pub struct RingBuffer<T> {
    inner: Arc<ArrayQueue<T>>,
}

impl<T> Clone for RingBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Buffer<T> for RingBuffer<T> {
    #[inline(always)]
    fn count(&self) -> u32 {
        self.inner.len() as u32
    }

    #[inline(always)]
    fn free(&self) -> u32 {
        (self.inner.capacity() - self.inner.len()) as u32
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    #[inline(always)]
    fn push(&mut self, value: T) -> Option<T> {
        self.inner.force_push(value)
    }
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let ring_buffer = ArrayQueue::<T>::new(capacity);

        Self {
            inner: Arc::new(ring_buffer),
        }
    }
}
