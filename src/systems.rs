use crate::player::Player;
use crate::components;
use crate::components::*;
use bevy::prelude::*;
use bevy::math::swizzles::Vec3Swizzles;

const MAX_CAMERA_SNAP_DISTANCE:f32 = 16.0f32; // If the camera is farther than this, just set it to the player.  This keeps a teleporting player from making the camera fly across the map.
const CAMERA_SMOOTHING:f32 = 0.001f32; // Should be greater than zero.

pub fn movement_system(
	time: Res<Time>,
	mut query: Query<(&mut Transform, &Velocity)>,
) {
	for (mut transform, velocity) in query.iter_mut() {
		let translation: &mut Vec3 = &mut transform.translation;
		translation.x += velocity.dx * time.delta_seconds();
		translation.y += velocity.dy * time.delta_seconds();
	}
}

pub fn camera_follow_system(
	//mut query: Query<(&mut Transform, With<Camera2d>)>,
	mut transforms: ParamSet<(
		Query<(&mut Transform, With<Camera2d>)>,
		Query<(&Transform, With<Player>)>
	)>,
) {
	let mut player_transform: Vec3 = Vec3::splat(0.);
	for (player_tf, _) in transforms.p1().iter() {
		player_transform = player_tf.translation.clone();
	}

	for (mut camera_tf, _) in transforms.p0().iter_mut() {
		let delta: Vec3 = player_transform - camera_tf.translation;
		if delta.length_squared() > MAX_CAMERA_SNAP_DISTANCE {
			camera_tf.translation.x = player_transform.x;
			camera_tf.translation.y = player_transform.y;
			// Don't apply Z or we'll mess up layers.
		} else {
			// Lerp the camera.
			camera_tf.translation.x += delta.x*CAMERA_SMOOTHING;
			camera_tf.translation.y += delta.y*CAMERA_SMOOTHING;
		}
	}
}

pub fn knockback_system(
	time: Res<Time>,
	mut query: Query<(&mut Transform, &mut Knockback, &mut RigidBody)>,
) {

}

pub fn update_last_facing(
	mut query: Query<(&Velocity, &mut LastFacing)>
) {
	for (vel, mut facing) in query.iter_mut() {
		// If the velocity is nonzero, update the facing direction.
		match vel.direction() {
			components::Direction::None => (),
			_ => facing.0 = vel.direction(),
		}
	}
}

pub fn static_dynamic_collision_system(
	mut dynamic_bodies: Query<(&mut Transform, &RigidBody)>,
	static_bodies: Query<(&Transform, &StaticBody), Without<RigidBody>>,
) {
	// Lazy O(n^2) approach.  We should make a resource that we update on level load.
	for (static_body_transform, static_body) in static_bodies.iter() {
		for (mut dynamic_body_transform, dynamic_body) in dynamic_bodies.iter_mut() {
			let maybe_displacement = minimum_separating_axis(&static_body_transform.translation.xy(), &static_body.size, &dynamic_body_transform.translation.xy(), &dynamic_body.size);
			if let Some(displacement) = maybe_displacement {
				dynamic_body_transform.translation.x += displacement.x;
				dynamic_body_transform.translation.y += displacement.y;
			}
		}
	}
}

pub fn dynamic_dynamic_collision_system(
	mut query: Query<(&mut Transform, &RigidBody)>,
) {
	let mut combinations = query.iter_combinations_mut();
	while let Some([mut a, mut b]) = combinations.fetch_next() {

	}
}

/// Returns the minimum force that needs to be applied to 'B' to remove it from 'A'.
/// If A and B do not overlap, returns None.
fn minimum_separating_axis(center_a: &Vec2, size_a: &Vec2, center_b: &Vec2, size_b: &Vec2) -> Option<Vec2> {
	// TODO: Implement GJK-Collision or Minkowski-Minimum-Separating-Axis.
	// If the two axes do not overlap, None is returned.
	// For ergonomics, maybe we should always return a vec2?

	let halfsize_a = *size_a*0.5;
	let halfsize_b = *size_b*0.5;

	// Distance from a's right to b's left.
	let ab_left = (center_a.x + halfsize_a.x) - (center_b.x - halfsize_b.x);
	if ab_left < 0.0 {
		return None;
	}

	// Distance from a's left to b's right.
	let ab_right = (center_a.x - halfsize_a.x) - (center_b.x + halfsize_b.x);
	if ab_right > 0.0 {
		return None;
	}

	// Distance from a's top to b's bottom.
	let ab_top = (center_a.y + halfsize_a.y) - (center_b.y - halfsize_b.y);
	if ab_top < 0.0 {
		return None;
	}

	// Distance from a's bottom to b's top.
	let ab_bottom = (center_a.y - halfsize_a.y) - (center_b.y + halfsize_b.y);
	if ab_bottom > 0.0 {
		return None;
	}

	// We have an overlap.  Pick the smallest of the bunch.
	let min_separating_axis: [(f32, Vec2); 4] = [
		(ab_left.abs(), Vec2::new(ab_left, 0.0)),
		(ab_right.abs(), Vec2::new(ab_right, 0.0)),
		(ab_top.abs(), Vec2::new(0.0, ab_top)),
		(ab_bottom.abs(), Vec2::new(0.0, ab_bottom)),
	];

	Some(
		min_separating_axis.iter().fold((f32::INFINITY, Vec2::new(69.0, 420.0)), |a, &b|{
			if a.0 <= b.0 {
				a
			} else {
				b
			}
		}).1
	)
}

#[cfg(test)]
mod tests {
	use bevy::math::Vec2;
	use super::minimum_separating_axis;

	#[test]
	fn test_minimum_separating_axis() {
		// Start A at the center and make it 1 unit wide.
		let mut a = Vec2::new(0.0, 0.0);
		let mut a_size = Vec2::new(1.0, 1.0);

		// Put be at (1, 1) and make it two units large.
		let mut b = Vec2::new(1.0, 1.0);
		let mut b_size = Vec2::new(2.0, 2.0);
		// Should overlap:
		assert!(minimum_separating_axis(&a, &a_size, &b, &b_size).is_some());

		// Now make b tiny.  It should not overlap.
		b_size.x = 0.1;
		b_size.y = 0.1;
		assert!(minimum_separating_axis(&a, &a_size, &b, &b_size).is_none());

		// Put B on the right and make A really tall.
		//   a-----------------------|
		//               |-----------b
		// (0,0) + + + + + + + + + (1,0)
		a_size.x = 2.0;
		a_size.y = 100.0;
		b.x = 1.0;
		b.y = 0.0;
		b_size.x = 1.0;
		b_size.y = 1.0;
		// It should push to the right.
		let mut collision = minimum_separating_axis(&a, &a_size, &b, &b_size);
		assert!(collision.is_some());
		assert_eq!(collision.unwrap().x, 0.5f32);
		assert!(collision.unwrap().x > collision.unwrap().y);

		// Put B to the left of A.
		b.x = -1.0;
		collision = minimum_separating_axis(&a, &a_size, &b, &b_size);
		assert!(collision.is_some());
		assert_eq!(collision.unwrap().x, -0.5f32);
	}
}