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
const PLAYER_SPEED: f32 = 40.0;
const PLAYER_MAX_PUSH_DISTANCE_SQUARED: f32 = 15.0f32 * 15.0f32;
const PLAYER_PUSH_DURATION_MS: u64 = 800;
const PLAYER_ATTACK_COOLDOWN_MS: u64 = 100;
const PLAYER_ANIMATION_FRAME_TIME: u64 = 200;
const ANIM_TILE_SIZE: f32 = 16.0;
const PLAYER_NUM_DIRECTIONS: usize = 4;
const PLAYER_FRAMES_PER_ANIMATION: usize = 4;
const PLAYER_NUM_ANIMATION_STATES: usize = 5;

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
		app.add_system(broadcast_player_death);
		app.add_system(player_keyboard_event_system);
		app.add_system(player_animation_system);
		//app.add_system_to_stage("player_init", respawn_player);
	}
}

// Player-specific components:

#[derive(Component)]
pub struct Player {
	pub max_speed: f32,
	pub attack_cooldown: Timer,

	pub last_frame_timer: Timer,
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
	let player_spritesheet_atlas = TextureAtlas::from_grid(player_spritesheet_handle, Vec2::splat(ANIM_TILE_SIZE), PLAYER_FRAMES_PER_ANIMATION, PLAYER_NUM_ANIMATION_STATES*PLAYER_NUM_DIRECTIONS);
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
		.insert(LastFacing(components::Direction::Down))
		.insert(YSort { base_layer: PLAYER_RENDER_PRIORITY })
		.insert(RigidBody {
			mass: 1.0,
			drag: 0.0,
			size: Vec2::splat(PLAYER_SIZE),
			layers: PhysicsLayer::ACTOR,
		})
		.insert(Player {
			max_speed: PLAYER_SPEED,
			attack_cooldown: Timer::new(Duration::from_millis(PLAYER_ATTACK_COOLDOWN_MS), false),
			last_frame_timer: Timer::new(Duration::from_millis(100), true),
			sprite_atlas_index: 0
		});
}

fn broadcast_player_death(
	mut ev_playerdeath: EventWriter<PlayerDeathEvent>,
	mut query: Query<Entity, (With<Player>, With<Dead>)>,
) {
	if let Ok(entity) = query.get_single() {
		// Player is dead.  :'(
		ev_playerdeath.send(PlayerDeathEvent(entity));
		//commands.entity(entity).despawn();
	}
}

fn player_animation_system(
	time: Res<Time>,
	mut query: Query<(&LastFacing, &Velocity, &mut TextureAtlasSprite, &mut Player)>,
	mut knockback_query: Query<&Knockback, With<Player>>,
	dead_query: Query<&Dead, With<Player>>,
) {
	let knockback_active = knockback_query.iter().count() > 0;
	let dead = dead_query.iter().count() > 0;

	if let Ok((facing, velocity, mut texture_atlas_sprite, mut player_state)) = query.get_single_mut() {
		// Major index: State.  Minor index: direction.  Min index: frame num.

		// Idle: 0  (*4 directions *4 frames per direction)
		// Walk: 1
		// Push: 2
		// Hit: 3
		// Dead: 4

		let mut major_frame_offset = PLAYER_FRAMES_PER_ANIMATION*PLAYER_NUM_DIRECTIONS;
		if dead {
			major_frame_offset *= 4;
		} else if knockback_active {
			major_frame_offset *= 3;
		} else if !player_state.attack_cooldown.finished() {
			major_frame_offset *= 2;
		} else if velocity.direction() != components::Direction::None {
			// Player is moving.  Use this or the last facing direction.
			major_frame_offset *= 1;
		} else {
			major_frame_offset = 0;
		}
		let direction_frame_offset = PLAYER_FRAMES_PER_ANIMATION * match facing.0 {
			components::Direction::Right => 0,
			components::Direction::Up => 1,
			components::Direction::Left => 2,
			components::Direction::Down => 3,
			_ => 0,
		};
		let mut frame_step = texture_atlas_sprite.index % PLAYER_FRAMES_PER_ANIMATION;

		player_state.last_frame_timer.tick(time.delta());
		if player_state.last_frame_timer.just_finished() {
			frame_step = (frame_step + 1)%PLAYER_FRAMES_PER_ANIMATION;
		}

		// Set our current frame based on this.
		texture_atlas_sprite.index = major_frame_offset + direction_frame_offset + frame_step;
	}
}

fn player_attack_system(
	mut commands: Commands,
	time: Res<Time>,
	kb: Res<Input<KeyCode>>,
	//game_textures: Res<GameTextures>,
	mut player_query: Query<(&Transform, &LastFacing, &mut Player)>,
	mut enemy_query: Query<(&Transform, &RigidBody, Entity), Without<Player>>,
) {
	if let Ok((player_tf, player_facing, mut player_state)) = player_query.get_single_mut() {
		// Decrease the attack cooldown if it's set.
		player_state.attack_cooldown.tick(time.delta());

		// Open question: do we want to push in the direction we last moved or allow full control?

		if kb.just_pressed(KeyCode::Space) && player_state.attack_cooldown.finished() {
			let player_xy:Vec2 = Vec2::new(player_tf.translation.x, player_tf.translation.y);
			player_state.attack_cooldown.reset();
			//let (x, y) = (player_tf.translation.x, player_tf.translation.y);

			//let mut spawn_attack = |: f32| {
			// Spawn a push effect _immediately_ for kickback.
			/*
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
			*/

			// Go through all the enemies and if they're close, give them a push.
			for (enemy_tf, enemy_rb, entity) in enemy_query.iter() {
				let enemy_xy:Vec2 = Vec2::new(enemy_tf.translation.x, enemy_tf.translation.y);
				let delta_position = player_xy - enemy_xy;
				if delta_position.length_squared() > PLAYER_MAX_PUSH_DISTANCE_SQUARED {
					continue;
				}
				let player_forward = match player_facing.0 {
					components::Direction::Right => Vec2::new(1.0, 0.0),
					components::Direction::Up => Vec2::new(0.0, 1.0),
					components::Direction::Left => Vec2::new(-1.0, 0.0),
					_ => Vec2::new(0.0, -1.0),
				};
				// If the dot product of the player_forward and the difference in positions is negative, the player is facing the enemy.
				if player_forward.dot(delta_position) >= 0.0f32 {
					continue;
				}
				// We are close enough and facing enemies.
				commands.entity(entity).insert(Knockback {
					impulse: player_forward,
					duration: Timer::new(Duration::from_millis(PLAYER_PUSH_DURATION_MS), false)
				});
			}
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