mod renderer_with_cpu_usage;
mod stream_manager;

use std::sync::{ Arc, Mutex };

use renderer_with_cpu_usage::RendererWithCpuUsage;
use ringbuf::{ HeapRb, Cons as Consumer, consumer::Consumer as _ };
use stream_manager::{StreamManager, StreamManagerController};

use crate::backend::{Backend, Renderer};
use cpal::{
	traits::{DeviceTrait, HostTrait},
	BufferSize, Device, StreamConfig,
};

use super::{CpalBackendSettings, Error};

enum State {
	Empty,
	Uninitialized {
		device: Device,
		config: StreamConfig,
	},
	Initialized {
		stream_manager_controller: StreamManagerController,
	},
}

/// A backend that uses [cpal](https://crates.io/crates/cpal) to
/// connect a [`Renderer`] to the operating system's audio driver.
pub struct CpalBackend {
	state: State,
	/// Whether the device was specified by the user.
	custom_device: bool,
	buffer_size: BufferSize,
	cpu_usage_consumer: Option<Mutex<Consumer<Arc<HeapRb<f32>>>>>,
}

impl CpalBackend {
	/**
	Returns the oldest reported CPU usage in the queue.

	The formula for the CPU usage is time elapsed / time allotted, where
	- time elapsed is the amount of time it took to fill the audio buffer
	  requested by the OS
	- time allotted is the maximum amount of time Kira could take to process
	  audio and still finish in time to avoid audio stuttering (num frames / sample
	  rate)
	*/
	pub fn pop_cpu_usage(&mut self) -> Option<f32> {
		self.cpu_usage_consumer
			.as_mut()
			.unwrap()
			.get_mut()
			.unwrap()
			.try_pop()
	}
}

impl Backend for CpalBackend {
	type Settings = CpalBackendSettings;

	type Error = Error;

	fn setup(
		settings: Self::Settings,
		_internal_buffer_size: usize,
	) -> Result<(Self, u32), Self::Error> {
		let host = cpal::default_host();

		let (device, custom_device) = if let Some(device) = settings.device {
			(device, true)
		} else {
			(
				host.default_output_device()
					.ok_or(Error::NoDefaultOutputDevice)?,
				false,
			)
		};

		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate.0;
		Ok((
			Self {
				state: State::Uninitialized { device, config },
				custom_device,
				buffer_size: settings.buffer_size,
				cpu_usage_consumer: None,
			},
			sample_rate,
		))
	}

	fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error> {
		let state = std::mem::replace(&mut self.state, State::Empty);
		if let State::Uninitialized { device, config } = state {
			let (renderer, cpu_usage_consumer) = RendererWithCpuUsage::new(renderer);
			self.state = State::Initialized {
				stream_manager_controller: StreamManager::start(
					renderer,
					device,
					config,
					self.custom_device,
					self.buffer_size,
				),
			};
			self.cpu_usage_consumer = Some(Mutex::new(cpu_usage_consumer));
		} else {
			panic!("Cannot initialize the backend multiple times")
		}
		Ok(())
	}
}

impl Drop for CpalBackend {
	fn drop(&mut self) {
		if let State::Initialized {
			stream_manager_controller,
		} = &self.state
		{
			stream_manager_controller.stop();
		}
	}
}
