use bevy::prelude::*;
use bevy::render::render_resource::TextureSampleType;
use bevy::render::texture::{ImageSampler, ImageSettings};
use bevy_ecs_ldtk::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

mod components;
mod level;
mod player;
mod resources;
mod systems;

const WINDOW_TITLE: &str = "Bevy Jam 2";
const TITLE_SCREEN: &str = "title.png";
const TILESET_RESOURCE: &str = "tileset.png";

fn main() {
	App::new()
		.insert_resource(ImageSettings::default_nearest())
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
		.add_startup_system(setup_system)
		// Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
		// or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
		.add_system(ui_example)
		.add_system(systems::movement_system)
		.add_system(systems::update_last_facing)
		.add_system(systems::camera_follow_system)
		.add_system(systems::static_dynamic_collision_system)
		.add_system(systems::dynamic_dynamic_collision_system)
		.add_system(systems::knockback_system)
		.add_plugin(level::LevelPlugin)
		.add_plugin(player::PlayerPlugin)
		.run();
}

fn setup_system(
		mut commands: Commands,
		mut asset_server: ResMut<AssetServer>,
) {
	// Camera
	let mut camera = Camera2dBundle::default();
	camera.projection.scale /= 4.0;  // Make things 4x bigger.  16x16 tiles -> 64x64.
	commands.spawn_bundle(camera);


	// 1280 wide screen / 64 pixel tiles -> 20 tiles wide
	//let texture_handle = asset_server.load(TILESET_RESOURCE);
	//let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4);
	//let tileset = texture_atlases.add(texture_atlas);

	// Add the sprite sheet resource.
	commands.insert_resource(resources::SpriteSheets {
		title_screen: asset_server.load(TITLE_SCREEN),
	});

	// Map is loaded in the setup system in the level.
}

fn ui_example(
	mut egui_context: ResMut<EguiContext>
) {
	egui::Window::new("Debug").show(egui_context.ctx_mut(), |ui| {
		ui.label("world");
	});
}