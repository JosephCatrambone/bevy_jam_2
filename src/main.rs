use crate::systems::movement_system;
use bevy::prelude::*;
use bevy::render::texture::ImageSampler;
use bevy_ecs_ldtk::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

mod actors;
mod components;
mod resources;
mod systems;

const WINDOW_TITLE: &str = "Bevy Jam 2";
const TITLE_SCREEN: &str = "title.png";
const PLAYER_SPRITESHEET: &str = "player.png";
const TILESET_RESOURCE: &str = "tileset.png";

fn main() {
	App::new()
		.insert_resource(ImageSampler::nearest_descriptor())
		.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
		.insert_resource(WindowDescriptor {
			title: WINDOW_TITLE.to_string(),
			width: 1280.0,
			height: 720.0,
			..Default::default()
		})
		.insert_resource(resources::GamePauseMode::default())
		.add_plugins(DefaultPlugins)
		.add_plugin(EguiPlugin)
		.add_plugin(LdtkPlugin)
		.add_startup_system(setup_system)
		// Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
		// or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
		.add_system(ui_example)
		.add_system(movement_system)
		.add_plugin(actors::player::PlayerPlugin)
		.register_ldtk_entity::<resources::LevelDoor>("Door")
		.insert_resource(LevelSelection::Index(0))
		.run();
}

fn setup_system(
		mut commands: Commands,
		mut asset_server: ResMut<AssetServer>,
		mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
	// Camera
	let mut camera = Camera2dBundle::default();
	camera.projection.scale /= 4.0;  // Make things 4x bigger.  16x16 tiles -> 64x64.
	commands.spawn_bundle(camera);

	// Load player spritesheet.
	let player_spritesheet_handle = asset_server.load(PLAYER_SPRITESHEET);
	let player_spritesheet_atlas = TextureAtlas::from_grid(player_spritesheet_handle, Vec2::new(64.0, 64.0), 10, 10);
	let player = texture_atlases.add(player_spritesheet_atlas);

	// 1280 wide screen / 64 pixel tiles -> 20 tiles wide
	let texture_handle = asset_server.load(TILESET_RESOURCE);
	let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4);
	let tileset = texture_atlases.add(texture_atlas);

	// Add the sprite sheet resource.
	commands.insert_resource(resources::SpriteSheets {
		title_screen: asset_server.load(TITLE_SCREEN),
		player: player,
		tileset: tileset,
	});

	// Load the map.
	commands.spawn_bundle(LdtkWorldBundle {
		ldtk_handle: asset_server.load("maps.ldtk"),
		..Default::default()
	});
}

fn ui_example(mut egui_context: ResMut<EguiContext>) {
	egui::Window::new("Debug").show(egui_context.ctx_mut(), |ui| {
		ui.label("world");
	});
}