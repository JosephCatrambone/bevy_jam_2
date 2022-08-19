use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

const WINDOW_TITLE: &str = "Bevy Jam 2";
const TITLE_SCREEN: &str = "title.png";
const PLAYER_SPRITESHEET: &str = "player.png";
const TILESET_RESOURCE: &str = "tileset.png";

struct GameTextures {
	title_screen: Handle<Image>,
	player: Handle<TextureAtlas>,
	tileset: Handle<TextureAtlas>,
}

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
		.insert_resource(WindowDescriptor {
			title: WINDOW_TITLE.to_string(),
			width: 1280.0,
			height: 720.0,
			..Default::default()
		})
		.add_plugins(DefaultPlugins)
		.add_plugin(EguiPlugin)
		.add_startup_system(setup_system)
		// Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
		// or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
		.add_system(ui_example)
		.run();
}

fn setup_system(
		mut commands: Commands,
		mut asset_server: ResMut<AssetServer>,
		mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
	// Camera
	commands.spawn_bundle(Camera2dBundle::default());

	// Load player spritesheet.
	let player_spritesheet_handle = asset_server.load(PLAYER_SPRITESHEET);
	let player_spritesheet_atlas = TextureAtlas::from_grid(player_spritesheet_handle, Vec2::new(64.0, 64.0), 12, 12);
	let player = texture_atlases.add(player_spritesheet_atlas);

	// 1280 wide screen / 64 pixel tiles -> 20 tiles wide
	let texture_handle = asset_server.load(TILESET_RESOURCE);
	let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4);
	let tileset = texture_atlases.add(texture_atlas);

	commands.insert_resource(GameTextures {
		title_screen: asset_server.load(TITLE_SCREEN),
		player: player,
		tileset: tileset,
	})
}

fn ui_example(mut egui_context: ResMut<EguiContext>) {
	egui::Window::new("Debug").show(egui_context.ctx_mut(), |ui| {
		ui.label("world");
	});
}