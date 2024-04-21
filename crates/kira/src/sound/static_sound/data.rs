#[cfg(feature = "symphonia")]
mod from_file;

#[cfg(test)]
mod test;

use std::{
	fmt::{Debug, Formatter},
	sync::Arc,
	time::Duration,
};

use crate::{
	dsp::Frame,
	sound::{
		EndPosition, IntoOptionalRegion, PlaybackPosition, PlaybackRate, Region, Sound, SoundData,
	},
	tween::{Tween, Value},
	OutputDestination, StartTime, Volume,
};

use super::{
	command_writers_and_readers, handle::StaticSoundHandle, sound::StaticSound, StaticSoundSettings,
};

/// A piece of audio loaded into memory all at once.
///
/// These can be cheaply cloned, as the audio data is shared
/// among all clones.
#[derive(Clone, PartialEq)]
pub struct StaticSoundData {
	/// The sample rate of the audio (in Hz).
	pub sample_rate: u32,
	/// The raw samples that make up the audio.
	pub frames: Arc<[Frame]>,
	/// Settings for the sound.
	pub settings: StaticSoundSettings,
	pub slice: Option<(usize, usize)>,
}

impl StaticSoundData {
	/**
	Sets when the sound should start playing.

	This returns a cheap clone of the `StaticSoundData` with the modified start time.

	# Examples

	Configuring a sound to start 4 ticks after a clock's current time:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		sound::static_sound::{StaticSoundData, StaticSoundSettings},
		clock::ClockSpeed,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let clock_handle = manager.add_clock(ClockSpeed::TicksPerMinute(120.0))?;
	let sound = StaticSoundData::from_file("sound.ogg")?
		.start_time(clock_handle.time() + 4);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn start_time(&self, start_time: impl Into<StartTime>) -> Self {
		let mut new = self.clone();
		new.settings.start_time = start_time.into();
		new
	}

	/// Sets where in the sound playback should start.
	///
	/// This returns a cheap clone of the `StaticSoundData` with the modified start position.
	pub fn start_position(&self, start_position: impl Into<PlaybackPosition>) -> Self {
		let mut new = self.clone();
		new.settings.start_position = start_position.into();
		new
	}

	/// Sets whether the sound should be played in reverse.
	///
	/// This returns a cheap clone of the `StaticSoundData` with the modified setting.
	pub fn reverse(&self, reverse: bool) -> Self {
		let mut new = self.clone();
		new.settings.reverse = reverse;
		new
	}

	/**
	Sets the portion of the sound that should be looped.

	This returns a cheap clone of the `StaticSoundData` with the modified loop region.

	# Examples

	Configure a sound to loop the portion from 3 seconds in to the end:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	let settings = StaticSoundSettings::new().loop_region(3.0..);
	```

	Configure a sound to loop the portion from 2 to 4 seconds:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	let settings = StaticSoundSettings::new().loop_region(2.0..4.0);
	```
	*/
	pub fn loop_region(&self, loop_region: impl IntoOptionalRegion) -> Self {
		let mut new = self.clone();
		new.settings.loop_region = loop_region.into_optional_region();
		new
	}

	/**
	Sets the volume of the sound.

	This returns a cheap clone of the `StaticSoundData` with the modified volume.

	# Examples

	Set the volume as a factor:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	let settings = StaticSoundSettings::new().volume(0.5);
	```

	Set the volume as a gain in decibels:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	let settings = StaticSoundSettings::new().volume(kira::Volume::Decibels(-6.0));
	```

	Link the volume to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::static_sound::{StaticSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let settings = StaticSoundSettings::new().volume(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn volume(&self, volume: impl Into<Value<Volume>>) -> Self {
		let mut new = self.clone();
		new.settings.volume = volume.into();
		new
	}

	/**
	Sets the playback rate of the sound.

	Changing the playback rate will change both the speed and the pitch of the sound.

	This returns a cheap clone of the `StaticSoundData` with the modified playback rate.

	# Examples

	Set the playback rate as a factor:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	let settings = StaticSoundSettings::new().playback_rate(0.5);
	```

	Set the playback rate as a change in semitones:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	use kira::sound::PlaybackRate;
	let settings = StaticSoundSettings::new().playback_rate(PlaybackRate::Semitones(-2.0));
	```

	Link the playback rate to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::static_sound::{StaticSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let settings = StaticSoundSettings::new().playback_rate(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn playback_rate(&self, playback_rate: impl Into<Value<PlaybackRate>>) -> Self {
		let mut new = self.clone();
		new.settings.playback_rate = playback_rate.into();
		new
	}

	/**
	Sets the panning of the sound, where 0 is hard left and 1 is hard right.

	This returns a cheap clone of the `StaticSoundData` with the modified panning.

	# Examples

	Set the panning to a static value:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	let settings = StaticSoundSettings::new().panning(0.25);
	```

	Link the panning to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::static_sound::{StaticSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.25,
	})?;
	let settings = StaticSoundSettings::new().panning(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn panning(&self, panning: impl Into<Value<f64>>) -> Self {
		let mut new = self.clone();
		new.settings.panning = panning.into();
		new
	}

	/**
	Sets the destination that this sound should be routed to.

	This returns a cheap clone of the `StaticSoundData` with the modified output destination.

	# Examples

	Set the output destination of a sound to a mixer track:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		track::TrackBuilder,
		sound::static_sound::{StaticSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let sub_track = manager.add_sub_track(TrackBuilder::new())?;
	let settings = StaticSoundSettings::new().output_destination(&sub_track);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set the output destination of a sound to an emitter in a spatial scene:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		spatial::{scene::SpatialSceneSettings, emitter::EmitterSettings},
		sound::static_sound::{StaticSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let mut scene = manager.add_spatial_scene(SpatialSceneSettings::default())?;
	let emitter = scene.add_emitter(mint::Vector3 {
		x: 0.0,
		y: 0.0,
		z: 0.0,
	}, EmitterSettings::default())?;
	let settings = StaticSoundSettings::new().output_destination(&emitter);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn output_destination(&self, output_destination: impl Into<OutputDestination>) -> Self {
		let mut new = self.clone();
		new.settings.output_destination = output_destination.into();
		new
	}

	/// Sets the tween used to fade in the sound from silence.
	///
	/// This returns a cheap clone of the `StaticSoundData` with the modified fade in tween.
	pub fn fade_in_tween(&self, fade_in_tween: impl Into<Option<Tween>>) -> Self {
		let mut new = self.clone();
		new.settings.fade_in_tween = fade_in_tween.into();
		new
	}

	/// Returns a cheap clone of the `StaticSoundData` with the specified settings.
	pub fn with_settings(&self, settings: StaticSoundSettings) -> Self {
		Self {
			settings,
			..self.clone()
		}
	}

	pub fn num_frames(&self) -> usize {
		num_frames(&self.frames, self.slice)
	}

	/// Returns the duration of the audio.
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.num_frames() as f64 / self.sample_rate as f64)
	}

	pub fn frame_at_index(&self, index: usize) -> Option<Frame> {
		frame_at_index(index, &self.frames, self.slice)
	}

	pub fn slice(&self, region: impl IntoOptionalRegion) -> Self {
		let mut new = self.clone();
		new.slice = region.into_optional_region().map(|Region { start, end }| {
			let start = start.into_samples(self.sample_rate);
			let end = match end {
				EndPosition::EndOfAudio => self.frames.len(),
				EndPosition::Custom(end) => end.into_samples(self.sample_rate),
			};
			(start, end)
		});
		new
	}

	pub(super) fn split(self) -> (StaticSound, StaticSoundHandle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		let sound = StaticSound::new(self, command_readers);
		let shared = sound.shared();
		(
			sound,
			StaticSoundHandle {
				command_writers,
				shared,
			},
		)
	}
}

impl SoundData for StaticSoundData {
	type Error = ();

	type Handle = StaticSoundHandle;

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		let (sound, handle) = self.split();
		Ok((Box::new(sound), handle))
	}
}

impl Debug for StaticSoundData {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("StaticSoundData")
			.field("sample_rate", &self.sample_rate)
			.field(
				"frames",
				&FramesDebug {
					len: self.frames.len(),
				},
			)
			.field("settings", &self.settings)
			.finish()
	}
}

struct FramesDebug {
	len: usize,
}

impl Debug for FramesDebug {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("[{} frames]", self.len))
	}
}

pub(crate) fn num_frames(frames: &[Frame], slice: Option<(usize, usize)>) -> usize {
	if let Some((start, end)) = slice {
		end - start
	} else {
		frames.len()
	}
}

pub(crate) fn frame_at_index(
	index: usize,
	frames: &[Frame],
	slice: Option<(usize, usize)>,
) -> Option<Frame> {
	if index >= num_frames(frames, slice) {
		return None;
	}
	let start = slice.map(|(start, _)| start).unwrap_or_default();
	Some(frames[index + start])
}
