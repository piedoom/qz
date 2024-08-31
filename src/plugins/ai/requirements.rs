use crate::prelude::*;
use bevy::prelude::*;
use bevy_htnp::prelude::*;

pub(super) fn enemies_in_view(mut in_range: Query<(&InRange, &mut HtnAgentWorld)>) {
    for (in_range, mut world_state) in in_range.iter_mut() {
        world_state.0.insert(
            Requirement::EnemiesInView.name(),
            !in_range.enemies.is_empty(),
        );
    }
}

pub(super) fn target_in_weapons_range(
    mut in_range: Query<(Entity, &InRange, &Equipped, &mut HtnAgentWorld)>,
    transforms: Query<&GlobalTransform>,
    weapons_query: Query<&Weapon>,
) {
    for (entity, in_range, equipped, mut world_state) in in_range.iter_mut() {
        // TODO: get first weapon range

        // let maybe_weapons = equipped.equipped.get(&EquipmentTypeId::Weapon);
        // match maybe_weapons {
        //     Some(weapons) => weapons_query.get(weapons.iter().next()).map(|weapon| weapon,
        //     None => todo!(),
        // }
        if let Some(enemy) = in_range.enemies.first() {
            let transform = transforms.get(entity).unwrap().compute_transform();
            let enemy_transform = transforms.get(*enemy).unwrap().compute_transform();

            let in_weapons_range = transform
                .translation
                .distance_squared(enemy_transform.translation)
                <= 4f32.powi(2);
            world_state
                .0
                .insert(Requirement::TargetInWeaponsRange.name(), in_weapons_range);
        }
    }
}
