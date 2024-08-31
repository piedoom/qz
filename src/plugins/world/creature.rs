use crate::prelude::*;
use avian3d::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_htnp::{
    data::{Requirements, WorldState},
    prelude::{HtnAgent, HtnAgentWorld},
    tasks::Task,
};
use bevy_turborand::prelude::*;

pub(super) fn on_spawn_creature(
    trigger: Trigger<triggers::SpawnCreature>,
    mut cmd: Commands,
    mut rng: ResMut<GlobalRng>,
    library: Res<Library>,
    creatures: Res<Assets<Creature>>,
    crafts: Res<Assets<Craft>>,
    items: Res<Assets<Item>>,
) {
    let triggers::SpawnCreature {
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
        task::Search,
        Waypoint::None,
    ));

    if let Some(spawner) = spawner {
        ent.insert((SpawnedFrom(*spawner),));
    }

    let credits = rng.usize(credits.0..=credits.1);
    if credits != 0 {
        ent.insert(Credits::new(credits));
    }

    let mut agent = HtnAgent::new();

    agent.add_task(Task::primitive(task::Search::NAME));
    agent.add_task(Task::primitive(task::Attack::NAME));
    agent.add_task(Task::primitive(task::Persue::NAME));

    agent.add_goal(
        "Destroy enemy",
        Requirements::new()
            .req_equals(Requirement::TargetDestroyed.name(), true)
            .build(),
        1.0,
    );

    let default_world_state = WorldState::new()
        .add(Requirement::EnemiesInView.name(), false)
        .add(Requirement::TargetInWeaponsRange.name(), false)
        .add(Requirement::TargetDestroyed.name(), false)
        .build();

    ent.insert((agent, HtnAgentWorld(default_world_state)));
}
