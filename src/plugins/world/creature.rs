use crate::prelude::*;
use avian3d::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_turborand::prelude::*;
use big_brain::prelude::*;

pub(super) fn on_spawn_creature(
    trigger: Trigger<trigger::SpawnCreature>,
    mut cmd: Commands,
    mut rng: ResMut<GlobalRng>,
    library: Res<Library>,
    creatures: Res<Assets<Creature>>,
    crafts: Res<Assets<Craft>>,
    items: Res<Assets<Item>>,
) {
    let trigger::SpawnCreature {
        name,
        translation,
        rotation,
        alliegance,
        spawner,
    } = trigger.event();
    let creature = library.creature(name).unwrap();
    let Creature {
        name,
        craft,
        drops,
        inventory,
        equipped,
        range,
        credits,
        model,
    } = creatures.get(&creature).cloned().unwrap();
    let craft = library
        .crafts
        .get(&format!("crafts/{}.craft.ron", craft))
        .and_then(|craft| crafts.get(craft))
        .unwrap();
    let drops = drops
        .into_iter()
        .filter_map(|(drop_name, drop_rate)| {
            library
                .items
                .get(&format!("items/{}.ron", drop_name))
                .map(|item| (item.clone(), drop_rate))
        })
        .collect();
    let mut ent = cmd.spawn((
        CraftBundle {
            collider: Collider::sphere(craft.size * 0.5),
            mass: Mass(craft.mass),
            craft: craft.clone(),
            transform: Transform::z_from_parts(translation, rotation),
            alliegance: alliegance.clone(),
            inventory: Inventory::with_capacity(craft.capacity)
                .with_many_from_str(
                    inventory.into_iter().collect::<HashMap<String, usize>>(),
                    &items,
                    &library,
                )
                .unwrap(),
            equipped,
            ..default()
        },
        // Persistent,
        Model::new(library.model(&model).unwrap()),
        Drops(drops),
        InRange::new(range),
        Name::new(name),
    ));

    if let Some(spawner) = spawner {
        ent.insert((SpawnedFrom(*spawner),));
    }

    let credits = rng.usize(credits.0..=credits.1);
    if credits != 0 {
        ent.insert(Credits::new(credits));
    }

    ent.insert(
        Thinker::build()
            .picker(FirstToScore { threshold: 0.8 })
            .when(
                scorers::Facing,
                Concurrently::build()
                    .push(actions::Attack)
                    .push(actions::Persue),
            )
            .when(
                EvaluatingScorer::build(scorers::Facing, LinearEvaluator::new_inversed()),
                actions::Persue,
            )
            .otherwise(actions::Idle),
        // .when(
        //     scorers::Danger {
        //         radius: 3f32..=15f32,
        //     },
        //     actions::Retreat,
        // ),
    );
}
