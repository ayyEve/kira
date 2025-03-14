use std::sync::{
	atomic::{AtomicU32, Ordering},
	Arc,
};

use kira::{
	backend::mock::{MockBackend, MockBackendSettings},
	effect::{Effect, EffectBuilder},
	info::Info,
	track::TrackBuilder,
	AudioManager, AudioManagerSettings, Frame,
};

use ringbuf::{ Cons, Prod, HeapRb as RingBuffer, producer::Producer as _, consumer::Consumer as _ };
type Producer<T> = Prod<Arc<RingBuffer<T>>>;
type Consumer<T> = Cons<Arc<RingBuffer<T>>>;

struct TestEffect {
	sample_rate: Arc<AtomicU32>,
	dt_producer: Producer<f64>,
}

impl Effect for TestEffect {
	fn init(&mut self, sample_rate: u32, _internal_buffer_size: usize) {
		self.sample_rate.store(sample_rate, Ordering::SeqCst);
	}

	fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.sample_rate.store(sample_rate, Ordering::SeqCst);
	}

	fn process(&mut self, _input: &mut [Frame], dt: f64, _info: &Info) {
		self.dt_producer.try_push(dt).ok().unwrap();
	}
}

struct TestEffectHandle {
	sample_rate: Arc<AtomicU32>,
	dt_consumer: Consumer<f64>,
}

struct TestEffectBuilder;

impl EffectBuilder for TestEffectBuilder {
	type Handle = TestEffectHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {

		let rb = Arc::new(RingBuffer::new(100));
		let dt_producer = Producer::new(rb.clone());
		let dt_consumer = Consumer::new(rb.clone());
		let sample_rate = Arc::new(AtomicU32::new(0));
		(
			Box::new(TestEffect {
				sample_rate: sample_rate.clone(),
				dt_producer,
			}),
			TestEffectHandle {
				sample_rate,
				dt_consumer,
			},
		)
	}
}

#[test]
fn change_sample_rate() {
	let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings {
		backend_settings: MockBackendSettings { sample_rate: 100 },
		..Default::default()
	})
	.unwrap();
	let mut effect_handle;
	let _track = manager
		.add_sub_track({
			let mut builder = TrackBuilder::new();
			effect_handle = builder.add_effect(TestEffectBuilder);
			builder
		})
		.unwrap();
	let backend = manager.backend_mut();
	backend.on_start_processing();
	assert_eq!(effect_handle.sample_rate.load(Ordering::SeqCst), 100);
	backend.process();
	assert_eq!(effect_handle.dt_consumer.try_pop(), Some(1.0 / 100.0));
	backend.set_sample_rate(200);
	assert_eq!(effect_handle.sample_rate.load(Ordering::SeqCst), 200);
	backend.process();
	assert_eq!(effect_handle.dt_consumer.try_pop(), Some(1.0 / 200.0));
}
