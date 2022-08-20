use crate::SpriteSheets;
use crate::components::Health;
use bevy::prelude::*;
use bevy::time::FixedTimestep;
use rand::{Rng, thread_rng};

const PLAYER_RENDER_PRIORITY:f32 = 1.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		//app.add_startup_system(player_startup);
		app.add_system_set(
			SystemSet::new()
				.with_run_criteria(FixedTimestep::step(1.0))
				.with_system(respawn_player)
		);
		app.add_system(check_for_player_death);
	}
}

#[derive(Component)]
pub struct Player;

fn respawn_player(
	player_query: Query<With<Player>>,
	sprite_sheets: Res<SpriteSheets>,
	time: Res<Time>,
) {
	if let Some(_) = player_query.iter().next() {
		return; // Nothing to do.
	}


}

fn check_for_player_death(
	mut commands: Commands,
	query: Query<(Entity, &Health, With<Player>)>,
) {
	//let (entity, player_health, _) = query.single();
	if let Some((entity, player_health, _)) = query.iter().next() {
		if player_health.current <= 0 {
			// Player is dead.  :'(
			commands.entity(entity).despawn();
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

fn player_keyboard_event_system(
	kb: Res<Input<KeyCode>>,
	mut query: Query<&mut Velocity, With<Player>>,
) {
	if let Ok(mut velocity) = query.get_single_mut() {
		velocity.x = if kb.pressed(KeyCode::Left) {
			-1.
		} else if kb.pressed(KeyCode::Right) {
			1.
		} else {
			0.
		}
	}
}
 */