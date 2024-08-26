use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub(super) fn on_spawn_building(
    trigger: Trigger<trigger::SpawnBuilding>,
    mut cmd: Commands,
    mut rng: ResMut<GlobalRng>,
    library: Res<Library>,
    buildings: Res<Assets<Building>>,
    items: Res<Assets<Item>>,
) {
    let trigger::SpawnBuilding {
        name,
        translation,
        rotation,
        alliegance,
    } = trigger.event();
    let Building {
        name,
        mass,
        health,
        size,
        drops,
        inventory,
        inventory_space,
        equipped,
        spawner,
        store,
        credits,
        store_margin,
    } = library
        .building(name)
        .and_then(|building| buildings.get(building.id()))
        .unwrap()
        .clone();

    let mut entity = cmd.spawn((
        (
            Name::new(name.clone()),
            Persistent,
            Structure,
            Health::new(health),
            Damage::default(),
            RigidBody::Dynamic,
            Mass(mass),
            Collider::sphere(size * 0.5),
            alliegance.clone(),
            Inventory::with_capacity(inventory_space)
                .with_many_from_str(inventory.into_iter().collect(), &items, &library)
                .unwrap(),
            equipped,
            Drops(
                drops
                    .into_iter()
                    .filter_map(|(drop, rate)| library.item(drop).map(|x| (x, rate)))
                    .collect(),
            ),
            CollisionLayers {
                memberships: LayerMask::from([PhysicsCategory::Structure]),
                filters: LayerMask::from([PhysicsCategory::Weapon, PhysicsCategory::Structure]),
            },
            LockedAxes::ROTATION_LOCKED,
            Transform::z_from_parts(translation, rotation),
        ),
        (GlobalTransform::IDENTITY,),
    ));

    if let Some(spawner) = spawner {
        entity.insert((spawner,));
    }

    if let Some(credits) = credits {
        entity.insert(Credits::new(credits));
    }

    if let (Some(store), Some(margin)) = (store, store_margin) {
        entity.insert((
            Store {
                items: store
                    .into_iter()
                    .filter_map(|(item, (numerator, denominator))| {
                        // Add sale items in randomly
                        // TODO: this can be rerolled at will, when it should persist
                        let value = rng.f32();
                        let value_to_beat = 1f32 - (numerator as f32 / denominator as f32);
                        match value >= value_to_beat {
                            true => library.item(item),
                            false => None,
                        }
                    })
                    .collect(),
                margin,
            },
            Dockings::default(),
            Model::new(library.model("structures/station").unwrap()).with_offset(-Vec3::Y * 5f32),
        ));
    }
}
