use std::sync::Arc;

use crossbeam::queue::ArrayQueue;

pub trait Buffer<T> {
    /// Return the number of items in the buffer.
    fn count(&self) -> u32;

    /// Return the number of items that can be pushed before [`Buffer::push`]
    /// starts returning a value.
    fn free(&self) -> u32;

    /// Return [None] if the buffer is empty.
    fn pop(&self) -> Option<T>;

    /// Return a previous value if the buffer is full.
    fn push(&self, value: T) -> Option<T>;
}

pub struct DescriptorBuffer<T> {
    inner: Arc<ArrayQueue<T>>,
}

impl<T> Clone for DescriptorBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Buffer<T> for DescriptorBuffer<T> {
    #[inline(always)]
    fn count(&self) -> u32 {
        self.inner.len() as u32
    }

    #[inline(always)]
    fn free(&self) -> u32 {
        (self.inner.capacity() - self.inner.len()) as u32
    }

    #[inline(always)]
    fn pop(&self) -> Option<T> {
        self.inner.pop()
    }

    #[inline(always)]
    fn push(&self, value: T) -> Option<T> {
        self.inner.force_push(value)
    }
}

impl<T> DescriptorBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let ring_buffer = ArrayQueue::<T>::new(capacity);

        Self {
            inner: Arc::new(ring_buffer),
        }
    }
}
