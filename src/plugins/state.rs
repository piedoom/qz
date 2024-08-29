use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use avian3d::prelude::{
    AngularVelocity, Collider, ColliderConstructor, CollisionLayers, ExternalImpulse, Friction,
    LinearDamping, LinearVelocity, LockedAxes, Mass, RigidBody,
};
use bevy::{
    asset::{AssetPath, LoadState},
    core_pipeline::bloom::BloomSettings,
    ecs::{
        observer::ObserverState,
        query::{QueryData, QueryEntityError, ROQueryItem},
    },
    pbr::{
        CascadeShadowConfig, Cascades, CascadesVisibleEntities, NotShadowCaster, NotShadowReceiver,
        VolumetricLight,
    },
    prelude::*,
    render::primitives::CascadesFrusta,
    scene::DynamicEntity,
    tasks::IoTaskPool,
    window::PrimaryWindow,
};
use bevy_etcetera::Directories;
use leafwing_input_manager::{
    prelude::{InputMap, KeyboardVirtualAxis},
    InputManagerBundle,
};

use crate::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::Save>()
            .init_resource::<SavePath>()
            .add_systems(OnEnter(AppState::load_game()), (enter_load_game,))
            .add_systems(OnEnter(AppState::new_game()), (enter_new_game,))
            .add_systems(OnEnter(AppState::main()), (spawn_camera, finalize_player))
            .add_systems(
                Update,
                (continue_loading,).run_if(in_state(AppState::load_game())),
            )
            .add_systems(OnEnter(AppState::save_game()), (enter_save_game,));
    }
}

fn enter_save_game(world: &mut World) {
    let AppState::SaveGame { save_path } = world.resource::<State<AppState>>().get().clone() else {
        unreachable!()
    };

    let mut scene = DynamicScene::default();

    // Extract necessary resources
    scene
        .resources
        .push(Box::new(world.resource::<Factions>().clone()));

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

        if let Some(inventory) = w.get::<Inventory>().cloned() {
            dynamic_entity
                .components
                .push(Box::new(InventoryBuilder::from_output(inventory)));
        }

        if let Some(drops) = w.get::<Drops>().cloned() {
            dynamic_entity
                .components
                .push(Box::new(DropsBuilder::from_output(drops)));
        }

        w.extract::<Player>(&mut dynamic_entity);
        w.extract::<Transform>(&mut dynamic_entity);
        w.extract::<Name>(&mut dynamic_entity);
        w.extract::<Alliegance>(&mut dynamic_entity);
        w.extract::<Craft>(&mut dynamic_entity);
        w.extract::<RigidBody>(&mut dynamic_entity);
        w.extract::<Credits>(&mut dynamic_entity);
        w.extract::<Damage>(&mut dynamic_entity);
        w.extract::<Destroyed>(&mut dynamic_entity);
        w.extract::<Energy>(&mut dynamic_entity);
        w.extract::<Model>(&mut dynamic_entity);
        w.extract::<Structure>(&mut dynamic_entity);
        w.extract::<GlobalTransform>(&mut dynamic_entity);
        w.extract::<Health>(&mut dynamic_entity);
        w.extract::<Persistent>(&mut dynamic_entity);
        w.extract::<Spawner>(&mut dynamic_entity);
        w.extract::<Dockings>(&mut dynamic_entity);
        w.extract::<CollisionLayers>(&mut dynamic_entity);
        w.extract::<LinearVelocity>(&mut dynamic_entity);
        w.extract::<LockedAxes>(&mut dynamic_entity);
        w.extract::<Controller>(&mut dynamic_entity);
        w.extract::<ChestsInRange>(&mut dynamic_entity);
        w.extract::<DockInRange>(&mut dynamic_entity);
        w.extract::<Friction>(&mut dynamic_entity);
        w.extract::<SpotLight>(&mut dynamic_entity);
        w.extract::<PointLight>(&mut dynamic_entity);
        w.extract::<DirectionalLight>(&mut dynamic_entity);
        w.extract::<CascadesFrusta>(&mut dynamic_entity);
        w.extract::<Cascades>(&mut dynamic_entity);
        w.extract::<CascadeShadowConfig>(&mut dynamic_entity);
        w.extract::<CascadesVisibleEntities>(&mut dynamic_entity);
        w.extract::<InheritedVisibility>(&mut dynamic_entity);
        w.extract::<ViewVisibility>(&mut dynamic_entity);
        w.extract::<VolumetricLight>(&mut dynamic_entity);
        w.extract::<Mass>(&mut dynamic_entity);

        if !dynamic_entity.components.is_empty() {
            scene.entities.push(dynamic_entity);
        }
    }

    let serialized_scene = {
        let registry = world.resource::<AppTypeRegistry>().read();
        scene.serialize(&registry).unwrap()
    };

    // // Create save path if it doesn't already exist
    // create_dir_all(save_path.clone().0).ok();

    if let Ok(mut window) = world
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .get_single_mut(world)
    {
        window.title = save_path.clone().to_string_lossy().to_string();
    }

    *world.resource_mut::<SavePath>() = SavePath(Some(save_path.clone()));

    // Save this scene handle to disk
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(save_path.clone())
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();

    world
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Main);
}

/// Load an entire save game
fn enter_load_game(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    entities: Query<Entity, (Without<ObserverState>, Without<Window>)>,
    state: Res<State<AppState>>,
) {
    let AppState::LoadGame { path } = state.get() else {
        unreachable!()
    };

    // Clean up
    for entity in entities.iter() {
        cmd.entity(entity).despawn_recursive();
    }
    // Begin load the given path
    let scene = assets.load::<DynamicScene>(AssetPath::from_path(path));
    cmd.insert_resource(CurrentlyLoading(scene));
}

/// Wait for the loaded savegame to deserialize, then spawn the world
pub fn continue_loading(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
    loading: Res<CurrentlyLoading>,
    asset_server: Res<AssetServer>,
) {
    if LoadState::Loaded == asset_server.load_state(loading.0.id()) {
        // The asset is fully loaded. Spawn the scene then transition states
        cmd.spawn(DynamicSceneBundle {
            scene: loading.0.clone(),
            ..Default::default()
        });
        let AppState::LoadGame { path } = state.get() else {
            unreachable!()
        };
        cmd.insert_resource(SavePath(Some(path.clone())));
        next_state.set(AppState::Main);
    }
}

/// Add some special components to the player each time
pub fn finalize_player(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    library: Res<Library>,
    settings: Res<Assets<Settings>>,
    players: Query<Entity, With<Player>>,
) {
    let settings = settings.get(&library.settings).unwrap();
    for player_entity in players.iter() {
        cmd.entity(player_entity)
            .insert((
                ExternalImpulse::default(),
                AngularVelocity::default(),
                LinearDamping::default(),
            ))
            .insert(InputManagerBundle::with_map(
                InputMap::default()
                    .with_axis(
                        Action::Turn,
                        KeyboardVirtualAxis::new(
                            settings.controls.keyboard.left,
                            settings.controls.keyboard.right,
                        ),
                    )
                    .with_axis(
                        Action::Thrust,
                        KeyboardVirtualAxis::new(
                            settings.controls.keyboard.brake,
                            settings.controls.keyboard.thrust,
                        ),
                    )
                    .with(Action::Fire, settings.controls.keyboard.fire)
                    .with(Action::Take, settings.controls.keyboard.take)
                    .with(Action::Interact, settings.controls.keyboard.interact),
            ))
            .with_children(|cmd| {
                cmd.spawn((
                    MaterialMeshBundle {
                        transform: Transform::from_translation(Vec3::Y * -8f32),
                        mesh: meshes.add(Plane3d::default().mesh().size(100.0, 100.0)),
                        material: materials.add(BackgroundMaterial {
                            position: default(),
                        }),
                        ..default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
            });
    }
}

pub fn spawn_camera(mut cmd: Commands, camera: Query<(), With<Camera>>) {
    if camera.is_empty() {
        // Spawn camera
        cmd.spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                transform: Transform::from_xyz(0f32, -1f32, 16f32)
                    .looking_at(Vec3::splat(0f32), Dir3::Z),
                ..default()
            },
            // VolumetricFogSettings {
            //     density: 0.05,
            //     absorption: 0.03,
            //     ..Default::default()
            // },
            BloomSettings::OLD_SCHOOL,
            // FogSettings {
            //     color: Color::srgb(0.25, 0.25, 0.25),
            //     falloff: FogFalloff::Linear {
            //         start: 5.0,
            //         end: 20.0,
            //     },
            //     ..default()
            // },
        ));
    }
}

/// Create a new world
pub fn enter_new_game(
    mut cmd: Commands,
    mut save_path: ResMut<SavePath>,
    mut next_state: ResMut<NextState<AppState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut factions: ResMut<Factions>,
    directories: Res<Directories>,
    crafts: Res<Assets<Craft>>,
    library: Res<Library>,
    assets: Res<AssetServer>,
) {
    cmd.insert_resource(Chunks::new());
    // Set the new save path
    let save_name = format!(
        "{}-{}.save.ron",
        random_word::gen(random_word::Lang::En),
        random_word::gen(random_word::Lang::En)
    );
    **save_path = Some(directories.data_dir().join(save_name));
    cmd.trigger(triggers::GenerateChunks {
        // Spawn in a cross shape
        chunk_indicies: vec![
            (0, 0).into(),
            (0, 1).into(),
            (1, 0).into(),
            (-1, 0).into(),
            (-1, -0).into(),
        ],
    });

    let player_faction = factions
        .get_faction("player")
        .cloned()
        .unwrap_or_else(|| factions.register("player"));
    let enemy_faction = factions
        .get_faction("enemy")
        .cloned()
        .unwrap_or_else(|| factions.register("enemy"));
    let player_alliegance = Alliegance {
        faction: player_faction,
        allies: [player_faction].into(),
        enemies: [enemy_faction].into(),
    };
    // cmd.spawn(bevy::pbr::FogVolumeBundle {
    //     transform: Transform::from_scale(Vec3::splat(35.0)),
    //     ..default()
    // });

    // Spawn player
    cmd.spawn((
        Player(0), // TODO: handle IDs for multiplayer
        Persistent,
        Name::new("player"),
        ChestsInRange {
            chests: default(),
            range: 5f32,
        },
        DockInRange {
            dock: None,
            range: 5f32,
        },
        CraftBundle {
            craft: crafts.get(&library.craft("bev").unwrap()).unwrap().clone(),
            alliegance: player_alliegance.clone(),
            inventory: Inventory::default(),
            equipped: EquippedBuilder {
                equipped: [
                    "minireactor.generator",
                    "light_laser.weapon",
                    "autoweld.repair",
                    "ion.battery",
                    "ion.battery",
                    "iron.armor",
                    "iron.armor",
                ]
                .map(ToString::to_string)
                .into(),
                slots: [
                    (EquipmentTypeId::Weapon, 1),
                    (EquipmentTypeId::RepairBot, 1),
                    (EquipmentTypeId::Generator, 1),
                    (EquipmentTypeId::Battery, 3),
                    (EquipmentTypeId::Armor, 3),
                ]
                .into(),
            },
            ..default()
        },
        Model::new(library.model("crafts/pest").unwrap()),
    ));

    cmd.spawn((
        DirectionalLightBundle {
            transform: Transform::from_translation(Vec3::new(5f32, 5f32, 10f32))
                .looking_at(Vec3::ZERO, Vec3::Z),
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        Persistent,
        VolumetricLight,
    ));

    // Move to the main game state
    next_state.set(AppState::Main);
}

/// Local-ish resource
#[derive(Resource)]
struct CurrentlyLoading(pub Handle<DynamicScene>);
