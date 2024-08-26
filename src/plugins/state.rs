use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use avian3d::prelude::{Collider, ColliderConstructor, CollisionLayers, RigidBody};
use bevy::{
    asset::LoadState,
    ecs::query::{QueryData, QueryEntityError, ROQueryItem},
    prelude::*,
    scene::DynamicEntity,
    tasks::IoTaskPool,
};
use bevy_etcetera::Directories;

use crate::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::Save>()
            .add_systems(OnEnter(AppState::load_game()), (enter_game_load,))
            .add_systems(
                Update,
                continue_loading_game.run_if(in_state(AppState::load_game())),
            )
            .add_systems(
                OnEnter(AppState::transition_zone()),
                (
                    save_zone_state,
                    transition_zone_state.after(save_zone_state),
                ),
            )
            .add_systems(
                OnEnter(AppState::save_game()),
                (save_zone_state, enter_game_save.after(save_zone_state)),
            )
            .add_systems(OnEnter(AppState::load_zone()), (load_zone_state,))
            .add_systems(Update, move_to_gate);
    }
}

fn enter_game_save(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    save_game_name: Option<Res<SaveGameName>>,
    universe: Res<Universe>,
    universe_position: Res<UniversePosition>,
    directories: Res<Directories>,
    factions: Res<Factions>,
) {
    let mut universe_serialized = UniverseSerialized {
        end: universe.end.clone(),
        graph: default(),
    };
    let save_game_name = if let Some(save_game_name) = save_game_name {
        save_game_name.clone()
    } else {
        let save_name = SaveGameName::new();
        cmd.insert_resource(save_name.clone());
        save_name
    };

    for (_, _) in universe
        .graph
        .node_weights()
        .zip(universe.graph.node_indices())
    {
        universe_serialized.graph = universe.graph.map(
            |node, weight| {
                if weight.scene.is_some() {
                    ZoneSerialized {
                        name: weight.name.clone(),
                        depth: weight.depth,
                        scene: Some(format!("{}.zone.ron", node.index())),
                    }
                } else {
                    ZoneSerialized {
                        name: weight.name.clone(),
                        depth: weight.depth,
                        scene: None,
                    }
                }
            },
            |_, weight| *weight,
        );
    }

    let save_path = directories
        .data_dir()
        .join(save_game_name.0.clone())
        .join(save_game_name.0.clone())
        .with_extension("save.ron");
    let universe_position = *universe_position;

    let save_game_name = save_game_name.0.clone();

    let factions = factions.clone();

    IoTaskPool::get()
        .spawn(async move {
            let serialized_save = ron::to_string(&Save {
                universe: universe_serialized,
                universe_position,
                name: save_game_name,
                factions,
            })
            .unwrap();
            // Write the save data to file
            File::create(save_path)
                .and_then(|mut file| file.write(serialized_save.as_bytes()))
                .expect("Error while writing save to file");
        })
        .detach();

    next_state.set(AppState::Main);
}

#[derive(Resource)]
struct Loading {
    save: Handle<Save>,
    started_loading_zones: bool,
    zones: Vec<Handle<DynamicScene>>,
}

fn enter_game_load(mut cmd: Commands, assets: Res<AssetServer>, state: Res<State<AppState>>) {
    let AppState::LoadGame(path) = state.get() else {
        unreachable!()
    };

    let save = assets.load::<Save>(path);
    let loading = Loading {
        save,
        zones: vec![],
        started_loading_zones: false,
    };
    cmd.insert_resource(loading);
}

// Ensure everything is loaded before moving to the next state
fn continue_loading_game(
    mut cmd: Commands,
    mut loading: ResMut<Loading>,
    mut next_state: ResMut<NextState<AppState>>,
    directories: Res<Directories>,
    assets: Res<AssetServer>,
    saves: Res<Assets<Save>>,
) {
    // check if main file is done loading
    if let bevy::asset::LoadState::Loaded = assets.load_state(loading.save.id()) {
        let save = saves.get(&loading.save).unwrap();

        // loaded. check if this is the first time this is true, and if so, load zones
        if !loading.started_loading_zones {
            loading.started_loading_zones = true;

            cmd.insert_resource(save.factions.clone());
            cmd.insert_resource(SaveGameName(save.name.clone()));
            cmd.insert_resource(Universe {
                end: save.universe.end.clone(),
                graph: save.universe.graph.map(
                    |_, weight| Zone {
                        name: weight.name.clone(),
                        depth: weight.depth,
                        scene: weight.scene.as_ref().map(|scene| {
                            let handle = assets
                                .load(directories.data_dir().join(save.name.clone()).join(scene));
                            loading.zones.push(handle.clone());
                            handle
                        }),
                    },
                    |_, weight| *weight,
                ),
            });
        }

        if !loading
            .zones
            .iter()
            .any(|x| assets.get_load_state(x) != Some(LoadState::Loaded))
        {
            // all loaded, transition!
            let save = saves.get(&loading.save).unwrap();

            cmd.insert_resource(save.universe_position);

            next_state.set(AppState::LoadZone {
                load: save.universe_position.0,
                previous: None,
            });
        }
    }
}

fn save_zone_state(world: &mut World) {
    let mut scene = DynamicScene::default();

    let entities = world
        .archetypes()
        .iter()
        .flat_map(|a| a.entities().iter().map(|e| e.id()))
        .collect::<Vec<_>>();

    for entity in entities.into_iter() {
        struct W<'a> {
            world: &'a mut World,
            entity: Entity,
        }
        impl<'a> W<'a> {
            fn resource<T: Resource>(&self) -> Option<&T> {
                self.world.get_resource::<T>()
            }
            fn get<T: Component>(&mut self) -> Option<&T> {
                self.world.query::<&T>().get(self.world, self.entity).ok()
            }
            fn extract<T: Component + Reflect + Clone>(
                &mut self,
                dynamic_entity: &mut DynamicEntity,
            ) {
                if let Some(component) = self.get::<T>() {
                    dynamic_entity.components.push(Box::new(component.clone()));
                }
            }
            fn has<T: Component>(&mut self) -> bool {
                self.get::<T>().is_some()
            }
            fn query<T: QueryData>(
                &mut self,
                entity: Entity,
            ) -> Result<ROQueryItem<T>, QueryEntityError> {
                self.world.query::<T>().get(self.world, entity)
            }
        }
        let mut w = W { world, entity };

        // Skip certain entities from serialization
        if !w.has::<Persistent>() {
            continue;
        }

        // Extract potential components to serialize
        let mut dynamic_entity = DynamicEntity {
            entity,
            components: Vec::new(),
        };

        if let Some(equipped) = w.get::<Equipped>().cloned() {
            // Getting equipment is somewhat indirect since the individual `Equipment` entities
            // contain the name of the equipment
            let equipped_string = equipped
                .iter()
                .flat_map(|(_, entities)| {
                    entities
                        .iter()
                        .copied()
                        .filter_map(|entity| {
                            w.query::<&Handle<Item>>(entity)
                                .cloned()
                                .map(|handle| {
                                    handle
                                        .path()
                                        .unwrap()
                                        .to_string()
                                        .replace("items/", "")
                                        .replace(".ron", "")
                                })
                                .ok()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            dynamic_entity.components.push(Box::new(EquippedBuilder {
                equipped: equipped_string,
                slots: equipped.slots.into_iter().collect(),
            }));
        }
        #[allow(clippy::manual_map)]
        if let Some(collider) = w.get::<Collider>().cloned() {
            let shape = collider.shape();
            let maybe_collider = if let Some(ball) = shape.as_ball() {
                Some(ColliderConstructor::Sphere {
                    radius: ball.radius,
                })
            } else if let Some(cuboid) = shape.as_cuboid() {
                let length = cuboid.half_extents * 2f32;
                Some(ColliderConstructor::Cuboid {
                    x_length: length.x,
                    y_length: length.y,
                    z_length: length.z,
                })
            } else if let Some(cylinder) = shape.as_cylinder() {
                Some(ColliderConstructor::Cylinder {
                    radius: cylinder.radius,
                    height: cylinder.half_height * 2f32,
                })
            } else {
                None
            };

            if let Some(cc) = maybe_collider {
                dynamic_entity.components.push(Box::new(cc));
            }
        }

        w.extract::<Transform>(&mut dynamic_entity);
        w.extract::<Name>(&mut dynamic_entity);
        w.extract::<Alliegance>(&mut dynamic_entity);
        w.extract::<Craft>(&mut dynamic_entity);
        w.extract::<RigidBody>(&mut dynamic_entity);
        w.extract::<CollisionLayers>(&mut dynamic_entity);
        w.extract::<Credits>(&mut dynamic_entity);
        w.extract::<Damage>(&mut dynamic_entity);
        w.extract::<Destroyed>(&mut dynamic_entity);
        w.extract::<Energy>(&mut dynamic_entity);
        w.extract::<Model>(&mut dynamic_entity);
        w.extract::<Structure>(&mut dynamic_entity);
        w.extract::<GlobalTransform>(&mut dynamic_entity);
        w.extract::<Gate>(&mut dynamic_entity);
        w.extract::<Health>(&mut dynamic_entity);
        w.extract::<Persistent>(&mut dynamic_entity);
        w.extract::<Spawner>(&mut dynamic_entity);
        w.extract::<Dockings>(&mut dynamic_entity);

        // if let Some(inventory) = w.get::<Inventory>().cloned() {
        //     dynamic_entity.components.push(Box::new(inventory));
        // }
        // let drops = w.get::<Drops>().cloned().map(|drops| {
        //     drops
        //         .0
        //         .iter()
        //         .map(|(handle, v)| {
        //             (
        //                 // TODO: This is horrible
        //                 handle
        //                     .path()
        //                     .unwrap()
        //                     .to_string()
        //                     .replace("items/", "")
        //                     .replace(".ron", ""),
        //                 v.clone(),
        //             )
        //         })
        //         .collect()
        // });

        if !dynamic_entity.components.is_empty() {
            scene.entities.push(dynamic_entity);
        }
    }

    let current = world.resource::<UniversePosition>().get();

    // Insert the serialized scene into the universe
    let scenes = world.resource::<Assets<DynamicScene>>();

    // reserve a handle
    let handle = scenes.reserve_handle();

    let serialized_scene = {
        let registry = world.resource::<AppTypeRegistry>().read();
        scene.serialize(&registry).unwrap()
    };

    // Add the asset
    let mut scenes = world.resource_mut::<Assets<DynamicScene>>();
    scenes.insert(handle.id(), scene);

    // Assign the scene handle to this node
    let mut universe = world.resource_mut::<Universe>();
    let node_weight = universe.graph.node_weight_mut(current).unwrap();

    node_weight.scene = Some(handle);

    // If a save file exists,
    if let Some(save_name) = world.get_resource::<SaveGameName>().cloned() {
        let directories = world.resource::<Directories>();

        // Get a unique save path within this save
        let save_dir = directories.data_dir().join(save_name.0);

        // Create paths if they don't already exist
        create_dir_all(&save_dir).ok();

        let save_path = save_dir
            .join(current.index().to_string())
            .with_extension("zone.ron");

        // Save this scene handle to disk
        IoTaskPool::get()
            .spawn(async move {
                // Write the scene RON data to file
                File::create(save_path)
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            })
            .detach();
    }
}

fn transition_zone_state(world: &mut World) {
    // Change the world position
    let AppState::TransitionZone { load } = world
        .get_resource::<State<AppState>>()
        .unwrap()
        .get()
        .clone()
    else {
        unreachable!()
    };
    let node = world.resource_mut::<UniversePosition>().0;
    world.resource_mut::<UniversePosition>().0 = load;
    world
        .get_resource_mut::<NextState<AppState>>()
        .unwrap()
        .set(AppState::LoadZone {
            load,
            previous: Some(node),
        });
}

/// Transition from one node to another, or from the first node
fn load_zone_state(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    players: Query<Entity, With<Player>>,
    things: Query<Entity, (With<Collider>, Without<Player>)>,
    universe: Res<Universe>,
    state: Res<State<AppState>>,
) {
    let AppState::LoadZone { load, previous } = state.get() else {
        unreachable!()
    };

    dbg!(universe
        .graph
        .node_indices()
        .map(|x| x.index())
        .collect::<Vec<_>>());

    if let Some(zone) = universe.graph.node_weight(*load) {
        // Clean up
        for thing in things.iter() {
            cmd.entity(thing).despawn_recursive();
        }

        // dbg!(&zone.description);
        match &zone.scene {
            Some(scene) => {
                cmd.spawn(DynamicSceneBundle {
                    scene: scene.clone(),
                    ..Default::default()
                });
            }

            None => {
                // Generate new world
                cmd.trigger(trigger::SpawnZone { node: *load })
            }
        }

        if let Some(previous) = previous {
            // Move the player to the correct position when its available
            for player in players.iter() {
                cmd.entity(player).insert(MoveToGate(*previous));
            }
        }
    } else {
        panic!("attempted to load invalid node index {:?}", load);
    }
    next_state.set(AppState::Main);
}

// move the player to a gate
fn move_to_gate(
    mut cmd: Commands,
    mut player_transforms: Query<(Entity, &mut Transform, &MoveToGate), With<Player>>,
    gates: Query<(&Gate, &GlobalTransform), (Without<Player>, With<Transform>)>,
) {
    for (player_entity, mut player_transform, move_to_gate) in player_transforms.iter_mut() {
        dbg!(
            gates.iter().map(|x| x.0.destination()).collect::<Vec<_>>(),
            move_to_gate.0
        );
        if let Some((_, gate_transform)) =
            gates.iter().find(|g| g.0.destination() == move_to_gate.0)
        {
            player_transform.translation = gate_transform.compute_transform().translation;
            cmd.entity(player_entity).remove::<MoveToGate>();
        }
    }
}
