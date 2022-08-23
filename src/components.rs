use std::time::Duration;
use bevy::prelude::*;
use bevy::sprite::Rect;
use bitflags::bitflags;

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
pub struct LastFacing(pub Direction);

#[derive(Clone, Component)]
pub struct Velocity {
	pub dx: f32,
	pub dy: f32,
}

impl Velocity {
	pub fn direction(&self) -> Direction {
		match (self.dx.signum(), self.dy.signum()) {
			(1.0, _) => Direction::Right,
			(-1.0, _) => Direction::Left,
			(0.0, 1.0) => Direction::Up,
			(0.0, -1.0) => Direction::Down,
			_ => Direction::None,
		}
	}
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
	pub duration: Duration,
}