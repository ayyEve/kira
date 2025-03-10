use std::{
	ops::{Deref, DerefMut},
	time::Instant,
	sync::Arc,
};

use ringbuf::{ 
	Cons as Consumer, 
	Prod as Producer, 
	HeapRb as RingBuffer, 
	producer::Producer as _ 
};

use crate::backend::Renderer;

const CPU_USAGE_RINGBUFFER_CAPACITY: usize = 100;

pub struct RendererWithCpuUsage {
	renderer: Renderer,
	cpu_usage_producer: Producer<Arc<RingBuffer<f32>>>,
}

impl RendererWithCpuUsage {
	pub fn new(renderer: Renderer) -> (Self, Consumer<Arc<RingBuffer<f32>>>) {
		let buf = Arc::new(RingBuffer::new(CPU_USAGE_RINGBUFFER_CAPACITY));
		let cpu_usage_producer = Producer::new(buf.clone());
		let cpu_usage_consumer = Consumer::new(buf);

		(
			Self {
				renderer,
				cpu_usage_producer,
			},
			cpu_usage_consumer,
		)
	}

	pub fn process(&mut self, out: &mut [f32], num_channels: u16, sample_rate: u32) {
		let allotted_time = out.len() as f32 / num_channels as f32 / sample_rate as f32;
		let start_time = Instant::now();
		self.renderer.process(out, num_channels);
		let end_time = Instant::now();
		let process_duration = end_time - start_time;
		let cpu_usage = process_duration.as_secs_f32() / allotted_time;
		self.cpu_usage_producer.try_push(cpu_usage).ok();
	}
}

impl Deref for RendererWithCpuUsage {
	type Target = Renderer;

	fn deref(&self) -> &Self::Target {
		&self.renderer
	}
}

impl DerefMut for RendererWithCpuUsage {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.renderer
	}
}
