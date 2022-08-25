use std::time::Duration;
use bevy::prelude::*;
use bevy_ecs_ldtk::ldtk::FieldInstanceEntityReference;
use hashbrown::HashMap;

pub struct SpriteSheets {
	pub title_screen: Handle<Image>,
	//pub player: Handle<TextureAtlas>,
}

pub struct LevelTransition {
	pub fade_out: bool, // If not fade out and not fade in, done!
	pub fade_in: bool,

	// Fade out first, then switch levels, then fade in.
	pub fade_time: Timer,

	pub destination_level_iid: String,
	pub destination_entity_iid: String,
}

impl LevelTransition {
	pub fn new() -> Self {
		LevelTransition {
			fade_in: false,
			fade_out: false,
			fade_time: Timer::new(Duration::from_millis(200), false),
			destination_level_iid: String::new(),
			destination_entity_iid: String::new(),
		}
	}

	pub fn active(&self) -> bool {
		self.fade_in || self.fade_out
	}

	pub fn start_transition_to_target(&mut self, target:&FieldInstanceEntityReference) {
		self.fade_out = true;
		self.fade_in = false;
		self.fade_time = Timer::new(Duration::from_millis(200), false);
		self.destination_level_iid = target.level_iid.clone();
		self.destination_entity_iid = target.entity_iid.clone();
	}
}

#[derive(Default)]
pub struct GamePauseMode {
	pub menu_visible: bool,
	pub screen_transition: bool,
	pub dialog_active: bool,
}

impl GamePauseMode {
	/// A convenience method which returns true if there are any systems active which are causing a game pause.
	fn game_paused(&self) -> bool {
		self.menu_visible || self.screen_transition || self.dialog_active
	}
}

