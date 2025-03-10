use std::ops::{Deref, DerefMut};

use std::sync::Arc;
use ringbuf::{ Cons, Prod, HeapRb as RingBuffer, producer::Producer as _ };
type Producer<T> = Prod<Arc<RingBuffer<T>>>;
type Consumer<T> = Cons<Arc<RingBuffer<T>>>;

/// Wraps `T` so that when it's dropped, it gets sent
/// back through a thread channel.
///
/// This allows us to retrieve the data after a closure
/// that takes ownership of the data is dropped because of,
/// for instance, a cpal error.
pub struct SendOnDrop<T> {
	data: Option<T>,
	producer: Producer<T>,
}

impl<T> SendOnDrop<T> {
	pub fn new(data: T) -> (Self, Consumer<T>) {
		let buf = Arc::new(RingBuffer::new(1));
		let producer = Producer::new(buf.clone());
		let consumer = Consumer::new(buf);

		(
			Self {
				data: Some(data),
				producer,
			},
			consumer,
		)
	}
}

impl<T> Deref for SendOnDrop<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.data.as_ref().unwrap()
	}
}

impl<T> DerefMut for SendOnDrop<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.data.as_mut().unwrap()
	}
}

impl<T> Drop for SendOnDrop<T> {
	fn drop(&mut self) {
		self.producer
			.try_push(self.data.take().unwrap())
			.ok()
			.expect("send on drop producer full");
	}
}
