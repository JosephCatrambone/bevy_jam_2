use bevy::prelude::*;

pub enum Direction {
	None,
	Right,
	Up,
	Left,
	Down,
}

#[derive(Clone, Component)]
pub struct Health {
	pub current: i8,
	pub max: u8,
}

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