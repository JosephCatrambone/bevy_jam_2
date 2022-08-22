use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_ldtk::LdtkEntity;

pub struct SpriteSheets {
	pub title_screen: Handle<Image>,
	pub player: Handle<TextureAtlas>,
	pub tileset: Handle<TextureAtlas>,
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

// Map spawnable components:

#[derive(Component, Clone, Default)]
pub struct DestinationLevelName(String);

#[derive(Component, Clone, Default)]
pub struct DestinationTile((u32, u32));

#[derive(Bundle, Clone, Default)] // Can't auto derive LdtkEntity.
pub struct LevelDoor {
	destination_map: DestinationLevelName,
	destination_tile: DestinationTile,
	//#[sprite_sheet_bundle]
	//#[bundle]
	//sprite_bundle: SpriteSheetBundle,
}

impl LdtkEntity for LevelDoor {
	fn bundle_entity(
		entity_instance: &EntityInstance,
		_: &LayerInstance,
		_: Option<&Handle<Image>>,
		_: Option<&TilesetDefinition>,
		_: &AssetServer,
		_: &mut Assets<TextureAtlas>,
	) -> LevelDoor {
		dbg!(entity_instance);

		println!("EntityWithFields added, here are some facts:");
		for field_instance in &entity_instance.field_instances {
			println!(
				"    its {} {}",
				field_instance.identifier,
				explain_field(&field_instance.value)
			);
		}

		let mut sprite = Sprite {
			custom_size: Some(Vec2::splat(16.)),
			..Default::default()
		};
		if let Some(color_field) = entity_instance
			.field_instances
			.iter()
			.find(|f| f.identifier == *"Color")
		{
			if let FieldValue::Color(color) = color_field.value {
				sprite.color = color;
			}
		}

		LevelDoor {
			destination_map: DestinationLevelName("".to_string()),
			destination_tile: DestinationTile((0, 0))
		}
	}
}

fn explain_field(value: &FieldValue) -> String {
	match value {
		FieldValue::Int(Some(i)) => format!("has an integer of {}", i),
		FieldValue::Float(Some(f)) => format!("has a float of {}", f),
		FieldValue::Bool(b) => format!("is {}", b),
		FieldValue::String(Some(s)) => format!("says {}", s),
		FieldValue::Color(c) => format!("has the color {:?}", c),
		FieldValue::Enum(Some(e)) => format!("is the variant {}", e),
		FieldValue::FilePath(Some(f)) => format!("references {}", f),
		FieldValue::Point(Some(p)) => format!("is at ({}, {})", p.x, p.y),
		a => format!("is hard to explain: {:?}", a),
	}
}
