use crate::components;
use crate::components::*;
use crate::level::ENTITY_Z;
use bevy::prelude::*;
use rand::{RngCore, thread_rng};
use std::time::Duration;

// Constants:

const SLIME_SPRITESHEET: &str = "slime.png";
const SLIME_RENDER_PRIORITY: f32 = ENTITY_Z;
const SIZE: f32 = 14.0;
const SPEED: f32 = 40.0;
const ATTACK_COOLDOWN_MS: u64 = 100;
const ANIMATION_FRAME_TIME: u64 = 200;
const ANIM_TILE_SIZE: f32 = 32.0;
const NUM_DIRECTIONS: usize = 4;
const FRAMES_PER_ANIMATION: usize = 4;
const NUM_ANIMATION_STATES: usize = 5;

// Plugin/Setup:

pub struct SlimePlugin;

impl Plugin for SlimePlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(slime_startup_system);
		app.add_system(slime_animation_system);
		app.add_system(slime_ai_system);
		//app.add_system_to_stage("player_init", respawn_player);
	}
}

// Player-specific components:

#[derive(Component)]
pub struct Slime {
	pub max_speed: f32,
	pub attack_cooldown: Timer,
	pub last_frame_timer: Timer,
	pub sprite_atlas_index: usize,
}

// Resources

pub struct SlimeSpriteSheet {
	pub spritesheet_handle: Handle<TextureAtlas>,
}

// Systems and methods:

fn slime_startup_system(
	mut commands: Commands,
	asset_server: ResMut<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
	// Load player spritesheet.
	let slime_spritesheet_asset_handle = asset_server.load(SLIME_SPRITESHEET);
	let spritesheet_atlas = TextureAtlas::from_grid(slime_spritesheet_asset_handle, Vec2::splat(ANIM_TILE_SIZE), FRAMES_PER_ANIMATION, NUM_ANIMATION_STATES*NUM_DIRECTIONS);
	let spritesheet = texture_atlases.add(spritesheet_atlas);

	commands.insert_resource(SlimeSpriteSheet {
		//texture_handle: SLIME_SPRITESHEET_handle,
		//texture_atlas: SLIME_SPRITESHEET_atlas,
		spritesheet_handle: spritesheet,
	});
}

pub fn spawn_slime(
	commands: &mut Commands,
	spritesheet: &Res<SlimeSpriteSheet>,
	pos: Vec2,
	tint: Color,
) {
	let mut rng = thread_rng();
	let mut ssb = SpriteSheetBundle {
		sprite: TextureAtlasSprite::new(0),
		texture_atlas: spritesheet.spritesheet_handle.clone(),
		transform: Transform {
			translation: Vec3::new(pos.x, pos.y, SLIME_RENDER_PRIORITY),
			rotation: Default::default(),
			scale: Vec3::new(1., 1., 1.)
		},
		global_transform: Default::default(),
		visibility: Default::default(),
		computed_visibility: Default::default()
	};
	ssb.sprite.color = tint;

	let mut anim_frame_timer = Timer::new(Duration::from_millis(ANIMATION_FRAME_TIME), true);
	anim_frame_timer.set_elapsed(Duration::from_millis(rng.next_u64() % ANIMATION_FRAME_TIME));

	commands
		.spawn_bundle(ssb)
		.insert(Health { max: 3, current: 3 })
		.insert(Velocity { dx: 0.0, dy: 0.0 })
		.insert(LastFacing(components::Direction::Down))
		.insert(YSort { base_layer: SLIME_RENDER_PRIORITY })
		.insert(RigidBody {
			mass: 1.0,
			drag: 0.0,
			size: Vec2::splat(SIZE),
			layers: PhysicsLayer::ACTOR,
		})
		.insert(Slime {
			max_speed: SPEED,
			attack_cooldown: Timer::new(Duration::from_millis(ATTACK_COOLDOWN_MS), false),
			last_frame_timer: anim_frame_timer,
			sprite_atlas_index: 0
		});
}

fn slime_ai_system(
	time: Res<Time>,
	mut query: Query<(&LastFacing, &Velocity, &mut Slime), Without<Dead>>,
) {
	for (_, _, mut slime) in query.iter_mut() {
		slime.attack_cooldown.tick(time.delta());
	}
}

fn slime_animation_system(
	time: Res<Time>,
	mut query: Query<(&LastFacing, Option<&Dead>, Option<&Knockback>, &Velocity, &mut TextureAtlasSprite, &mut Slime)>,
) {
	for (facing, maybe_dead, maybe_hit, velocity, mut texture_atlas_sprite, mut state) in query.iter_mut() {
		let hit = maybe_hit.is_some();
		let dead = maybe_dead.is_some();
		// Major index: State.  Minor index: direction.  Min index: frame num.

		// Idle: 0  (*4 directions *4 frames per direction)
		// Walk: 1
		// Push: 2
		// Hit: 3
		// Dead: 4

		let mut major_frame_offset = FRAMES_PER_ANIMATION*NUM_DIRECTIONS;
		if dead {
			major_frame_offset *= 4;
		} else if hit {
			major_frame_offset *= 3;
		} else if !state.attack_cooldown.finished() {
			major_frame_offset *= 2;
		} else if velocity.direction() != components::Direction::None {
			// Player is moving.  Use this or the last facing direction.
			major_frame_offset *= 1;
		} else {
			major_frame_offset = 0;
		}
		let direction_frame_offset = FRAMES_PER_ANIMATION * match facing.0 {
			components::Direction::Right => 0,
			components::Direction::Up => 1,
			components::Direction::Left => 2,
			components::Direction::Down => 3,
			_ => 0,
		};
		let mut frame_step = texture_atlas_sprite.index % FRAMES_PER_ANIMATION;

		state.last_frame_timer.tick(time.delta());
		if state.last_frame_timer.just_finished() {
			frame_step = (frame_step + 1)%FRAMES_PER_ANIMATION;
		}

		// Set our current frame based on this.
		texture_atlas_sprite.index = major_frame_offset + direction_frame_offset + frame_step;
	}
}
