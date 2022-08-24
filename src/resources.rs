use bevy::prelude::*;

pub struct SpriteSheets {
	pub title_screen: Handle<Image>,
	//pub player: Handle<TextureAtlas>,
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

