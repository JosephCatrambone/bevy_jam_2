use crate::resources;
use crate::resources::*;
use crate::components;
use crate::components::*;
use crate::level::ENTITY_Z;
use bevy::prelude::*;
use bevy::time::FixedTimestep;
use rand::{Rng, thread_rng};

// Constants:

const PLAYER_RENDER_PRIORITY:f32 = ENTITY_Z;

// Plugin/Setup:

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(add_player_state);
		//app.add_system_set_to_stage(
		app.add_system_set(
			SystemSet::new()
				.with_run_criteria(FixedTimestep::step(1.0))
				.with_system(respawn_player),
		);
		app.add_system(check_for_player_death);
		app.add_system(player_keyboard_event_system);
	}
}

// Player-specific components:

#[derive(Component)]
pub struct Player;

// Resources

pub struct PlayerStart(Vec2);  // Used if a player happens to fall outside of the map.

pub struct PlayerState {
	pub max_speed: f32,
	pub dead: bool,
}

impl Default for PlayerState {
	fn default() -> Self {
		PlayerState {
			max_speed: 100f32,
			dead: false,
		}
	}
}

pub struct PlayerDeathEvent(Entity);

// Systems and methods:

fn add_player_state(
	mut commands: Commands
) {
	// We should consider loading this from a saved state or config file.
	commands.insert_resource(PlayerState::default());
}

fn respawn_player(
	mut commands: Commands,
	player_query: Query<With<Player>>,
	sprite_sheets: Res<SpriteSheets>,
) {
	if let Some(_) = player_query.iter().next() {
		return; // Nothing to do.
	}

	let mut ssb = SpriteSheetBundle {
		sprite: TextureAtlasSprite::new(0),
		texture_atlas: sprite_sheets.player.clone(),
		transform: Transform {
			translation: Vec3::new(0., 0., PLAYER_RENDER_PRIORITY),
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
			size: Vec2::new(8.0, 8.0),
			layers: PhysicsLayer::ACTOR,
		})
		.insert(Player);
}

pub fn player_oob_system(
	oob_target: Res<PlayerStart>,
	query: Query<(&mut Transform, With<Player>)>,
) {
	// If the player somehow ends up out-of-bounds,
}

fn check_for_player_death(
	//mut commands: Commands,
	//mut ev_playerdeath: EventWriter<PlayerDeathEvent>,
	query: Query<(Entity, &Health, With<Player>)>,
) {
	//let (entity, player_health, _) = query.single();
	//let (entity, player_health, _) = query.single_mut();
	for (entity, player_health, _) in query.iter() {
		if player_health.current <= 0 {
			// Player is dead.  :'(
			//commands.entity(entity).despawn();
			//ev_playerdeath.send(PlayerDeathEvent(entity));
		}
	}
}
/*
fn player_fire_system(
	mut commands: Commands,
	kb: Res<Input<KeyCode>>,
	game_textures: Res<GameTextures>,
	query: Query<&Transform, With<Player>>,
) {
	if let Ok(player_tf) = query.get_single() {
		if kb.just_pressed(KeyCode::Space) {
			let (x, y) = (player_tf.translation.x, player_tf.translation.y);
			let x_offset = PLAYER_SIZE.0 / 2. * SPRITE_SCALE - 5.;

			let mut spawn_laser = |x_offset: f32| {
				commands
					.spawn_bundle(SpriteBundle {
						texture: game_textures.player_laser.clone(),
						transform: Transform {
							translation: Vec3::new(x + x_offset, y + 15., 0.),
							scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
							..Default::default()
						},
						..Default::default()
					})
					.insert(Laser)
					.insert(FromPlayer)
					.insert(SpriteSize::from(PLAYER_LASER_SIZE))
					.insert(Movable { auto_despawn: true })
					.insert(Velocity { x: 0., y: 1. });
			};

			spawn_laser(x_offset);
			spawn_laser(-x_offset);
		}
	}
}
*/

fn player_keyboard_event_system(
	kb: Res<Input<KeyCode>>,
	mut player_state: ResMut<PlayerState>,
	mut query: Query<&mut Velocity, With<Player>>,
) {
	if let Ok(mut velocity) = query.get_single_mut() {
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