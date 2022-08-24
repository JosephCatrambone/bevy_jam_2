use bevy::prelude::*;
use bitflags::bitflags;

#[derive(Eq, PartialEq)]
pub enum Direction {
	None,
	Right,
	Up,
	Left,
	Down,
}

bitflags! {
	#[derive(Component, Default)]
	pub struct PhysicsLayer: u32 {
		// const None =  0b00000000;   Per bitset library: Don't define 'none'.
		const WORLD = 0b00000001;
		const ACTOR = 0b00000010;
		const ALL = Self::WORLD.bits | Self::ACTOR.bits;
	}
}

#[derive(Clone, Component)]
pub struct Health {
	pub current: i8,
	pub max: u8,
}

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct LastFacing(pub Direction);

#[derive(Clone, Component)]
pub struct Velocity {
	pub dx: f32,
	pub dy: f32,
}

impl Velocity {
	/// direction is None when dx and dy are zero.
	pub fn direction(&self) -> Direction {
		let moving = self.dx.abs() > 0.01 || self.dy.abs() > 0.01;
		if !moving {
			return Direction::None;
		}
		let horizontal = self.dx.abs() > self.dy.abs();
		if horizontal {
			if self.dx > 0. {
				Direction::Right
			} else {
				Direction::Left
			}
		} else {
			if self.dy > 0. {
				Direction::Up
			} else {
				Direction::Down
			}
		}
	}
}

#[derive(Component)]
pub struct YSort {
	pub base_layer: f32,
}

// Transforms for these are separate.

#[derive(Clone, Component, Debug, Default)]
pub struct Area2d {
	pub size: Vec2,
	pub layers: PhysicsLayer,
}

#[derive(Clone, Component, Debug, Default)]
pub struct RigidBody {
	pub mass: f32,
	pub drag: f32,
	pub size: Vec2,
	pub layers: PhysicsLayer,
}

#[derive(Clone, Component, Debug, Default)]
pub struct StaticBody {
	pub size: Vec2,
	pub layers: PhysicsLayer,
}

#[derive(Clone, Component, Debug)]
pub struct Knockback {
	pub impulse: Vec2, // force = mass * acceleration.  impulse = mass * delta velocity = f_avg * delta t
	pub duration: Timer,
}