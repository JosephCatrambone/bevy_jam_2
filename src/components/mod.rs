use bevy::prelude::*;

#[derive(Clone, Component)]
pub struct Health {
	pub current: i8,
	pub max: u8,
}