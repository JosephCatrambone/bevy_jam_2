use std::time::Duration;
use crate::components::Area2d;
use crate::components::FadeOverlay;
use crate::components::PhysicsLayer;
use crate::components::RigidBody;
use crate::components::StaticBody;
use crate::player::{Player, PlayerRestartPosition};
use crate::resources::LevelTransition;
use crate::slime::{SlimeSpriteSheet, spawn_slime};
use crate::systems::minimum_separating_axis;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_ecs_ldtk::ldtk::FieldInstanceEntityReference;
use bevy_ecs_ldtk::prelude::*;
use hashbrown::HashMap;
use hashbrown::HashSet;
use crate::resources;

const FADE_Z: f32 = 10.0; // This should be above everything.
const OVERLAY_DECORATION_NAME: &str = "OBJECTS_TOP_DECO";
const OVERLAY_DECORATION_Z: f32 = 7.;
const OBJECT_TOP_NAME: &str = "OBJECTS_TOP";
const OBJECT_TOP_Z: f32 = 6.;
const ENTITY_NAME: &str = "ENTITIES";
pub const ENTITY_Z: f32 = 5.;
const OBJECT_DECORATION_NAME: &str = "OBJECTS_DECO";
const OBJECT_DECORATION_Z: f32 = 4.;
const OBJECT_NAME: &str = "OBJECTS";
const OBJECT_Z: f32 = 3.;
const GROUND_DECORATION_NAME: &str = "GROUND_DECO";
const GROUND_DECORATION_Z: f32 = 2.;
const GROUND_NAME: &str = "GROUND";
const GROUND_Z: f32 = 1.;
const COLLISION_LAYER_NAME: &str = "COLLISION";
const COLLISION_Z: f32 = -1.;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
	fn build(&self, app: &mut App) {
		//app.insert_resource(PlayerRestartPosition::default());
		//app.add_startup_system(add_player_state);
		//app.add_system(player_keyboard_event_system);
		app.add_plugin(LdtkPlugin);
		app.add_startup_system(setup_system);
		app.add_system(make_collision_object_system);
		app.add_system(process_spawned_level_entity_system);
		app.add_system(process_spawned_level_layers_system);
		app.add_system(level_door_interaction_system);
		app.add_system(level_transition_system);
		//.register_ldtk_int_cell::<level::WallBundle>(1) // This should match up with 'WALL' on the collision layer.
		app.register_ldtk_int_cell_for_layer::<WallBundle>(COLLISION_LAYER_NAME, 1); // This should match up with 'WALL' on the collision layer.
		app.register_ldtk_entity::<LevelDoor>("DOOR");
		app.insert_resource(LevelSelection::Index(0));
	}
}

fn setup_system(
	mut commands: Commands,
	asset_server: ResMut<AssetServer>,
) {
	// Set up a big rectangle to drawn on top of everything for level transitions.
	// If there's a better way to fade, figure that out.
	commands.spawn_bundle(SpriteBundle {
		sprite: Sprite {
			color: Color::rgba(0.0, 0.0, 0.0, 0.0),
			custom_size: Some(Vec2::new(8000.0, 8000.0)),
			..default()
		},
		transform: Transform::from_xyz(0.0, 0.0, FADE_Z),
		..default()
	}).insert(FadeOverlay);

	commands.insert_resource(resources::LevelTransition::new());

	// Load the map.
	let ldtk_world_map = LdtkWorldBundle {
		ldtk_handle: asset_server.load("maps.ldtk"),
		..Default::default()
	};
	commands.spawn_bundle(ldtk_world_map);
}

// Region -- Level Door Handling

#[derive(Clone, Component)]
struct DoorDestination(FieldInstanceEntityReference);

#[derive(Bundle, Clone)] // Can't auto derive LdtkEntity.
pub struct LevelDoor {
	transform: Transform, // We use only the translation, but this is important for consistency.
	trigger_volume: Area2d,
	destination: DoorDestination,
	//#[sprite_sheet_bundle]
	//#[bundle]
	//sprite_bundle: SpriteSheetBundle,
}

impl LdtkEntity for LevelDoor {
	fn bundle_entity(
		entity_instance: &EntityInstance,
		_: &LayerInstance,
		_: Option<&Handle<Image>>,
		_: Option<&TilesetDefinition>,
		_: &AssetServer,
		_: &mut Assets<TextureAtlas>,
	) -> LevelDoor {
		// Despite not being explicitly added as children, doors seem to get cleaned up after level changes.

		let mut destination_entity_ref:Option<FieldInstanceEntityReference> = None;
		// Extract "destination_name" from the fields:
		// It would be nice if we could have all of the named_locations in advance, but...
		if let Some(destination_field) = entity_instance
			.field_instances
			.iter()
			.find(|f| f.identifier == *"destination")
		{
			if let FieldValue::EntityRef(dest_ref) = &destination_field.value {
				destination_entity_ref = dest_ref.clone();
			}
		}

		if destination_entity_ref.is_none() {
			eprintln!("Level sanity check failed.  Missing reference in {}", &entity_instance.identifier);
		}

		let origin:Vec2 = Vec2::new(entity_instance.px.x as f32, entity_instance.px.y as f32);

		LevelDoor {
			transform: Transform::from_xyz(origin.x, origin.y, 0.0),
			trigger_volume: Area2d {
				size: Vec2::new(entity_instance.width as f32, entity_instance.height as f32),
				layers: PhysicsLayer::WORLD,
			},
			destination: DoorDestination(destination_entity_ref.expect("Map invariant violated: destination is null."))
		}
	}
}

fn level_door_interaction_system(
	mut transition: ResMut<LevelTransition>,
	door_query: Query<(&Transform, &Area2d, &DoorDestination), Without<Player>>,
	mut player_query: Query<(&mut Transform, &RigidBody), With<Player>>,
) {
	if transition.active() { return; }

	// Check for collisions between the player and doors.
	for (player_tf, player_body) in player_query.iter() {

		for (door_tf, door_area, door_dest) in door_query.iter() {
			if let Some(_) = minimum_separating_axis(&player_tf.translation.xy(), &player_body.size, &door_tf.translation.xy(), &door_area.size) {
				// We are touching a door!
				transition.start_transition_to_target(&door_dest.0);
				break;
			}
		}
	}
}

/// Handle a level transition.
/// If a level transition is active, perform a fade.
/// When the fade is complete, swap the level.
/// When the swap is complete, look for added entities and, if we find one that matches the target, move the player there.
/// When that's done, fade in.
/// When the final fade is done, deactivate the transition and clear the targets.
pub fn level_transition_system(
	time: Res<Time>,
	mut level: ResMut<LevelSelection>,
	mut transition: ResMut<LevelTransition>,
	mut fade_query: Query<&mut Sprite, With<FadeOverlay>>,
	entity_query: Query<(&Transform, &EntityInstance), Added<EntityInstance>>,
	mut player_query: Query<(&mut Transform, &Player), Without<EntityInstance>>,
) {
	if !transition.active() { return; }

	// Update the level transition.
	transition.fade_time.tick(time.delta());
	let percent_complete = transition.fade_time.percent();
	// Set the fade.
	if let Ok(mut fade) = fade_query.get_single_mut() {
		let alpha = if transition.fade_out {
			percent_complete
		} else {
			1.0 - percent_complete
		};
		fade.color = Color::rgba(0.0, 0.0, 0.0, alpha);
	}

	// Check if the destination is loaded.
	let destination_level_loaded:bool = if let LevelSelection::Iid(target_iid) = level.as_ref() {
		&transition.destination_level_iid == target_iid
	} else {
		false
	};

	// If we faded all the way out, start to fade in.
	if transition.fade_time.finished() && transition.fade_out {
		if !destination_level_loaded {
			*level = LevelSelection::Iid(transition.destination_level_iid.clone());
		}

		// Update our fade our or our fade in.
		// Transition in the other direction.
		transition.fade_time.reset();
		transition.fade_out = false;
		transition.fade_in = true; // Start fading in.  The next frame we should correctly move the player.
	}

	// If the levels are switched and we're just starting to fade in, try and set the player's location.
	// If we have faded out, check to see if the desired entities have been loaded.
	if transition.fade_in && destination_level_loaded {
		// Perform either a level swap or an entity search for the target.
		if let Ok((mut player_tf, _)) = player_query.get_single_mut() {
			for (transform, entity_instance) in entity_query.iter() {
				if entity_instance.iid == transition.destination_entity_iid {
					player_tf.translation.x = transform.translation.x;
					player_tf.translation.y = transform.translation.y;
				}
			}
		}
	}

	// Have we faded all the way in?  We're done!
	if transition.fade_time.finished() && transition.fade_in && destination_level_loaded {
		transition.fade_in = false;
		// Don't worry about swapping the fade colors.  The new fade method will do that.
	}
}

// Region END -- Level Door Handling

// Region -- Level Render Order Updates

// LDTK does not do any changes to world_depth, so ground does not render below objects.
// We could perhaps do this with a level offset in the editor?
pub fn process_spawned_level_layers_system(
	mut query: Query<(&mut Transform, &LayerMetadata), Added<LayerMetadata>>,
) {
	for (mut transform, layer) in query.iter_mut() {
		match layer.identifier.as_ref() {
			OVERLAY_DECORATION_NAME => transform.translation.z = OVERLAY_DECORATION_Z,
			OBJECT_DECORATION_NAME => transform.translation.z = OBJECT_DECORATION_Z,
			ENTITY_NAME => transform.translation.z = ENTITY_Z,
			OBJECT_TOP_NAME => transform.translation.z = OBJECT_TOP_Z,
			OBJECT_NAME => transform.translation.z = OBJECT_Z,
			GROUND_DECORATION_NAME => transform.translation.z = GROUND_DECORATION_Z,
			GROUND_NAME => transform.translation.z = GROUND_Z,
			COLLISION_LAYER_NAME => transform.translation.z = COLLISION_Z,
			_ => {
				eprintln!("Unidentified layer name: {}", &layer.identifier)
			}
		}
	}
}

// Region END -- Level Render Order Updates

// Region -- Level Collision

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
	wall: Wall,
}

// This is called when LDTK loader instances an entity.
// Better to use the .register_ldtk_entity::<resources::LevelDoor>("Door") method, but this is an option.
fn process_spawned_level_entity_system(
	mut commands: Commands,
	slime_sprite_sheet: Res<SlimeSpriteSheet>,
	mut player_start: ResMut<PlayerRestartPosition>,
	level_query: Query<(Entity, &Handle<LdtkLevel>)>, // Used to assign entities as children of the level.
	entity_query: Query<(&Transform, &EntityInstance), Added<EntityInstance>>,
) {
	let mut spawned_entities: Vec<Entity> = vec![];

	for (transform, entity_instance) in entity_query.iter() {
		if entity_instance.identifier == *"PLAYER_SPAWN" {
			player_start.position.x = transform.translation.x;
			player_start.position.y = transform.translation.y;
		}
		else if entity_instance.identifier == *"SLIME_SPAWN" {
			let color = if let Some(color_field) = entity_instance
				.field_instances
				.iter()
				.find(|f| f.identifier == *"color")
			{
				if let FieldValue::Color(color) = color_field.value {
					color
				} else {
					Color::rgb(1.0, 1.0, 1.0)
				}
			} else {
				Color::rgb(1.0, 1.0, 1.0)
			};

			spawned_entities.push(spawn_slime(
				&mut commands,
				&slime_sprite_sheet,
				Vec2::new(transform.translation.x, transform.translation.y),
				color
			));
		}
	}

	// Set all of the spawned entities as children of the current map.
	//let level = levels.get(level_handle).expect("Level should be loaded by this point");
	if let Ok((level_entity, _level_handle)) = level_query.get_single() {
		//commands.entity(level_entity).with_children(|level| { level.spawn(); });
		commands.entity(level_entity).push_children(&spawned_entities);
	};
}

/// Stolen from the LDTK platformer source:
/// https://github.com/Trouv/bevy_ecs_ldtk/blob/main/examples/platformer/systems.rs
///
/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn make_collision_object_system(
	mut commands: Commands,
	wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
	parent_query: Query<&Parent, Without<Wall>>,
	level_query: Query<(Entity, &Handle<LdtkLevel>)>,
	levels: Res<Assets<LdtkLevel>>,
) {
	/// Represents a wide wall that is 1 tile tall
	/// Used to spawn wall collisions
	#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
	struct Plate {
		left: i32,
		right: i32,
	}

	/// A simple rectangle type representing a wall of any size
	#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
	struct Rect {
		left: i32,
		right: i32,
		top: i32,
		bottom: i32,
	}

	// Consider where the walls are
	// storing them as GridCoords in a HashSet for quick, easy lookup
	//
	// The key of this map will be the entity of the level the wall belongs to.
	// This has two consequences in the resulting collision entities:
	// 1. it forces the walls to be split along level boundaries
	// 2. it lets us easily add the collision entities as children of the appropriate level entity
	let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

	wall_query.for_each(|(&grid_coords, parent)| {
		// An intgrid tile's direct parent will be a layer entity, not the level entity
		// To get the level entity, you need the tile's grandparent.
		// This is where parent_query comes in.
		if let Ok(grandparent) = parent_query.get(parent.get()) {
			level_to_wall_locations
				.entry(grandparent.get())
				.or_insert(HashSet::new())
				.insert(grid_coords);
		}
	});

	if !wall_query.is_empty() {
		level_query.for_each(|(level_entity, level_handle)| {
			if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
				let level = levels
					.get(level_handle)
					.expect("Level should be loaded by this point");

				let LayerInstance {
					c_wid: width,
					c_hei: height,
					grid_size,
					..
				} = level
					.level
					.layer_instances
					.clone()
					.expect("Level asset should have layers")[0];

				// combine wall tiles into flat "plates" in each individual row
				let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

				for y in 0..height {
					let mut row_plates: Vec<Plate> = Vec::new();
					let mut plate_start = None;

					// + 1 to the width so the algorithm "terminates" plates that touch the right
					// edge
					for x in 0..width + 1 {
						match (plate_start, level_walls.contains(&GridCoords { x, y })) {
							(Some(s), false) => {
								row_plates.push(Plate {
									left: s,
									right: x - 1,
								});
								plate_start = None;
							}
							(None, true) => plate_start = Some(x),
							_ => (),
						}
					}

					plate_stack.push(row_plates);
				}

				// combine "plates" into rectangles across multiple rows
				let mut wall_rects: Vec<Rect> = Vec::new();
				let mut previous_rects: HashMap<Plate, Rect> = HashMap::new();

				// an extra empty row so the algorithm "terminates" the rects that touch the top
				// edge
				plate_stack.push(Vec::new());

				for (y, row) in plate_stack.iter().enumerate() {
					let mut current_rects: HashMap<Plate, Rect> = HashMap::new();
					for plate in row {
						if let Some(previous_rect) = previous_rects.remove(plate) {
							current_rects.insert(
								*plate,
								Rect {
									top: previous_rect.top + 1,
									..previous_rect
								},
							);
						} else {
							current_rects.insert(
								*plate,
								Rect {
									bottom: y as i32,
									top: y as i32,
									left: plate.left,
									right: plate.right,
								},
							);
						}
					}

					// Any plates that weren't removed above have terminated
					wall_rects.append(&mut previous_rects.values().copied().collect());
					previous_rects = current_rects;
				}

				commands.entity(level_entity).with_children(|level| {
					// Spawn colliders for every rectangle..
					// Making the collider a child of the level serves two purposes:
					// 1. Adjusts the transforms to be relative to the level for free
					// 2. the colliders will be despawned automatically when levels unload
					for wall_rect in wall_rects {
						level
							.spawn()
							.insert(StaticBody {
								size: Vec2::new((((wall_rect.right+1)-wall_rect.left) * grid_size) as f32, (((wall_rect.top+1)-wall_rect.bottom) * grid_size) as f32),
								layers: PhysicsLayer::WORLD,
							})
							.insert(Transform::from_xyz(
								(wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32 / 2.,
								(wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32 / 2.,
								0.,
							))
							.insert(GlobalTransform::default());
					}
				});
			}
		});
	}
}

fn explain_field(value: &FieldValue) -> String {
	match value {
		FieldValue::Int(Some(i)) => format!("has an integer of {}", i),
		FieldValue::Float(Some(f)) => format!("has a float of {}", f),
		FieldValue::Bool(b) => format!("is {}", b),
		FieldValue::String(Some(s)) => format!("says {}", s),
		FieldValue::Color(c) => format!("has the color {:?}", c),
		FieldValue::Enum(Some(e)) => format!("is the variant {}", e),
		FieldValue::FilePath(Some(f)) => format!("references {}", f),
		FieldValue::Point(Some(p)) => format!("is at ({}, {})", p.x, p.y),
		a => format!("is hard to explain: {:?}", a),
	}
}

// Region END -- Level Collision