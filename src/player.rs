use crate::resources;
use crate::resources::*;
use crate::components;
use crate::components::*;
use crate::level::ENTITY_Z;
use bevy::prelude::*;
use bevy::time::FixedTimestep;
use rand::{Rng, thread_rng};
use std::time::Duration;

// Constants:

const PLAYER_SPRITESHEET: &str = "player.png";
const PLAYER_RENDER_PRIORITY: f32 = ENTITY_Z;
const PLAYER_SIZE: f32 = 14.0;
const PLAYER_SPEED: f32 = 30.0;
const PLAYER_ATTACK_COOLDOWN_MS: u64 = 100;

// Plugin/Setup:

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(PlayerRestartPosition::default());
		app.add_startup_system(player_startup_system);
		app.add_event::<PlayerDeathEvent>();
		// DEBUG: Player spawn after 1 sec.  In future, this is handled differently.
		app.add_system_set(SystemSet::new().with_run_criteria(FixedTimestep::step(1.0)).with_system(player_respawn_system),);
		app.add_system(player_attack_system);
		app.add_system(check_for_player_death);
		app.add_system(player_keyboard_event_system);
		//app.add_system_to_stage("player_init", respawn_player);
	}
}

// Player-specific components:

#[derive(Component)]
pub struct Player {
	pub max_speed: f32,
	pub dead: bool,
	pub attack_cooldown: Timer,
	pub sprite_atlas_index: usize,
}

// Resources

pub struct PlayerSpriteSheet {
	pub spritesheet_handle: Handle<TextureAtlas>,
}

//pub struct PlayerRestartPosition(Vec2);  // Used if a player happens to fall outside of the map.
#[derive(Default)]
pub struct PlayerRestartPosition {
	// Used if a player happens to fall outside of the map.
	pub position: Vec2,
	pub with_damage: i8,
}

pub struct PlayerDeathEvent(Entity);

// Systems and methods:

fn player_startup_system(
	mut commands: Commands,
	mut asset_server: ResMut<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
	// Load player spritesheet.
	let player_spritesheet_handle = asset_server.load(PLAYER_SPRITESHEET);
	let player_spritesheet_atlas = TextureAtlas::from_grid(player_spritesheet_handle, Vec2::new(16.0, 16.0), 2, 4);
	let player_spritesheet = texture_atlases.add(player_spritesheet_atlas);

	commands.insert_resource(PlayerSpriteSheet {
		//texture_handle: player_spritesheet_handle,
		//texture_atlas: player_spritesheet_atlas,
		spritesheet_handle: player_spritesheet,
	});
}

fn player_respawn_system(
	mut commands: Commands,
	start: Res<PlayerRestartPosition>,
	spritesheet: Res<PlayerSpriteSheet>,
	mut player_query: Query<&mut Player>,
) {
	if let Some(_) = player_query.iter().next() {
		return; // Nothing to do.
	}

	// TODO: Wait on spritesheet to be loaded.

	let mut ssb = SpriteSheetBundle {
		sprite: TextureAtlasSprite::new(0),
		texture_atlas: spritesheet.spritesheet_handle.clone(),
		transform: Transform {
			translation: Vec3::new(start.position.x, start.position.y, PLAYER_RENDER_PRIORITY),
			rotation: Default::default(),
			scale: Vec3::new(1., 1., 1.)
		},
		global_transform: Default::default(),
		visibility: Default::default(),
		computed_visibility: Default::default()
	};

	commands
		.spawn_bundle(ssb)
		.insert(Health { max: 3, current: 3 })
		.insert(Velocity { dx: 0.0, dy: 0.0 })
		.insert(RigidBody {
			mass: 1.0,
			size: Vec2::splat(PLAYER_SIZE),
			layers: PhysicsLayer::ACTOR,
		})
		.insert(Player {
			max_speed: PLAYER_SPEED,
			dead: false,
			attack_cooldown: Timer::new(Duration::from_millis(PLAYER_ATTACK_COOLDOWN_MS), false),
			sprite_atlas_index: 0
		});
}

fn check_for_player_death(
	//mut commands: Commands,
	mut ev_playerdeath: EventWriter<PlayerDeathEvent>,
	mut query: Query<(Entity, &Health, &mut Player)>,
) {
	//let (entity, player_health, _) = query.single();
	//let (entity, player_health, _) = query.single_mut();
	for (entity, player_health, mut player_state) in query.iter_mut() {
		if player_health.current <= 0 {
			// Player is dead.  :'(
			//commands.entity(entity).despawn();
			ev_playerdeath.send(PlayerDeathEvent(entity));
			player_state.dead = true;
		}
	}
}

fn player_animation_system(
	mut query: Query<(&mut TextureAtlasSprite, &mut Player)>,
) {

}

fn player_attack_system(
	mut commands: Commands,
	time: Res<Time>,
	kb: Res<Input<KeyCode>>,
	//game_textures: Res<GameTextures>,
	mut query: Query<(&Transform, &mut Player)>,
) {
	if let Ok((player_tf, mut player_state)) = query.get_single_mut() {
		// Decrease the attack cooldown if it's set.
		player_state.attack_cooldown.tick(time.delta());

		// Open question: do we want to push in the direction we last moved or allow full control?

		if kb.just_pressed(KeyCode::Space) && player_state.attack_cooldown.finished() {
			println!("Push!");
			player_state.attack_cooldown.reset();
			let (x, y) = (player_tf.translation.x, player_tf.translation.y);

			//let mut spawn_attack = |: f32| {
			// Spawn a push effect _immediately_ for kickback.
			commands
				.spawn_bundle(SpriteBundle {
					transform: Transform::from_xyz(x, y, PLAYER_RENDER_PRIORITY),
					..Default::default()
				})
				//.insert(Laser)
				//.insert(FromPlayer)
				//.insert(SpriteSize::from(PLAYER_LASER_SIZE))
				//.insert(Movable { auto_despawn: true })
				.insert(Velocity { dx: 0., dy: 1. });
		}
	}
}

fn player_keyboard_event_system(
	kb: Res<Input<KeyCode>>,
	mut query: Query<(&mut Velocity, &Player)>,
) {
	if let Ok((mut velocity, player_state)) = query.get_single_mut() {
		velocity.dx = 0.0;
		if kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A) {
			velocity.dx -= player_state.max_speed;
		}
		if kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D) {
			velocity.dx += player_state.max_speed;
		}

		velocity.dy = 0.0;
		if kb.pressed(KeyCode::Up) || kb.pressed(KeyCode::W) {
			velocity.dy += player_state.max_speed;
		}
		if kb.pressed(KeyCode::Down) || kb.pressed(KeyCode::S) {
			velocity.dy -= player_state.max_speed;
		}
	}
}

fn make_push_area(
	mut commands: Commands,
	direction: Vec2,

) {

}