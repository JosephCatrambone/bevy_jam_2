use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_ldtk::LdtkEntity;

pub struct SpriteSheets {
	pub title_screen: Handle<Image>,
	pub player: Handle<TextureAtlas>,
}

pub struct GamePauseMode {
	pub menu_visible: bool,
	pub screen_transition: bool,
	pub dialog_active: bool,
}

impl Default for GamePauseMode {
	fn default() -> Self {
		GamePauseMode {
			menu_visible: false,
			screen_transition: false,
			dialog_active: false,
		}
	}
}

impl GamePauseMode {
	/// A convenience method which returns true if there are any systems active which are causing a game pause.
	fn game_paused(&self) -> bool {
		self.menu_visible || self.screen_transition || self.dialog_active
	}
}

