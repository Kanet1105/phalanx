use std::sync::Arc;

use crossbeam::queue::ArrayQueue;

pub trait Buffer<T>: Send + Sync {
    fn new(size: usize) -> Self
    where
        Self: Sized;

    fn len(&self) -> u32;

    fn pop(&self) -> Option<T>;

    fn push(&self, value: T) -> Option<T>;
}

pub struct DescriptorBuffer<T>(Arc<ArrayQueue<T>>);

unsafe impl<T> Send for DescriptorBuffer<T> {}

unsafe impl<T> Sync for DescriptorBuffer<T> {}

impl<T> Clone for DescriptorBuffer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Buffer<T> for DescriptorBuffer<T> {
    fn new(size: usize) -> Self {
        let ring = ArrayQueue::<T>::new(size as usize);

        Self(Arc::new(ring))
    }

    fn len(&self) -> u32 {
        self.0.len() as u32
    }

    fn pop(&self) -> Option<T> {
        self.0.pop()
    }

    fn push(&self, value: T) -> Option<T> {
        self.0.force_push(value)
    }
}
