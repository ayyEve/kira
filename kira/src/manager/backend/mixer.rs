use flume::Sender;
use indexmap::IndexMap;

use crate::{
	command::MixerCommand,
	frame::Frame,
	mixer::{effect_slot::EffectSlot, SubTrackId, Track, TrackIndex, TrackSettings},
	parameter::Parameters,
};

pub(crate) struct Mixer {
	main_track: Track,
	sub_tracks: IndexMap<SubTrackId, Track>,
}

impl Mixer {
	pub fn new() -> Self {
		Self {
			main_track: Track::new(TrackSettings::default()),
			sub_tracks: IndexMap::new(),
		}
	}

	pub fn run_command(
		&mut self,
		command: MixerCommand,
		tracks_to_unload_sender: &mut Sender<Track>,
		effect_slots_to_unload_sender: &mut Sender<EffectSlot>,
	) {
		match command {
			MixerCommand::AddSubTrack(id, track) => {
				self.sub_tracks.insert(id, track);
			}
			MixerCommand::AddEffect(index, id, effect, settings) => {
				let track = match index {
					TrackIndex::Main => Some(&mut self.main_track),
					TrackIndex::Sub(id) => self.sub_tracks.get_mut(&id),
				};
				if let Some(track) = track {
					track.add_effect(id, effect, settings);
				}
			}
			MixerCommand::RemoveSubTrack(id) => {
				if let Some(track) = self.sub_tracks.remove(&id) {
					tracks_to_unload_sender.try_send(track).ok();
				}
			}
			MixerCommand::RemoveEffect(effect_id) => {
				let track = match effect_id.track_index() {
					TrackIndex::Main => Some(&mut self.main_track),
					TrackIndex::Sub(id) => self.sub_tracks.get_mut(&id),
				};
				if let Some(track) = track {
					if let Some(effect_slot) = track.remove_effect(effect_id) {
						effect_slots_to_unload_sender.try_send(effect_slot).ok();
					}
				}
			}
		}
	}

	pub fn add_input(&mut self, index: TrackIndex, input: Frame) {
		let track = match index {
			TrackIndex::Main => &mut self.main_track,
			TrackIndex::Sub(id) => self.sub_tracks.get_mut(&id).unwrap_or(&mut self.main_track),
		};
		track.add_input(input);
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters) -> Frame {
		let mut main_input = Frame::from_mono(0.0);
		for (_, sub_track) in &mut self.sub_tracks {
			main_input += sub_track.process(dt, parameters);
		}
		self.main_track.add_input(main_input);
		self.main_track.process(dt, parameters)
	}
}
