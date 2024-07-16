use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use events::{EquipEvent, InventoryEvent};
use rand::Rng;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InventoryEvent>()
            .add_event::<EquipEvent>()
            .add_systems(
                Update,
                (
                    manage_equipment.pipe(handle_errors::<InventoryError>),
                    manage_inventory.pipe(handle_errors::<InventoryError>),
                    init_equipment,
                    manage_drops.pipe(handle_errors::<InventoryError>),
                    update_chests_in_range,
                ),
            );
    }
}

fn manage_inventory(
    mut inventories: Query<&mut Inventory>,
    mut events: EventReader<events::InventoryEvent>,
) -> Result<(), InventoryError> {
    for event in events.read() {
        match event {
            InventoryEvent::Transfer {
                from,
                to,
                item,
                amount,
            } => {
                let [mut from, mut to] = inventories
                    .get_many_mut([*from, *to])
                    .map_err(|_| InventoryError::Unqueriable)?;
                from.transfer(item, &mut to, *amount)?;
            }
        }
    }
    Ok(())
}

fn manage_equipment(
    mut cmd: Commands,
    mut events: EventReader<events::EquipEvent>,
    mut inv_equip: Query<(&mut Inventory, &mut Equipment)>,
    children: Query<&Children>,
    items: Query<&Item>,
) -> Result<(), InventoryError> {
    for event in events.read() {
        match event {
            events::EquipEvent::Equip {
                entity,
                item,
                transfer_from_inventory,
            } => {
                // get the inventory and equipment components of the given entity
                let (mut inventory, mut equipment) = inv_equip
                    .get_mut(*entity)
                    .map_err(|_| InventoryError::Unqueriable)?;

                if *transfer_from_inventory {
                    // shuffle inventory
                    inventory.transfer(item, &mut equipment.inventory, Some(1))?;
                }

                // Add entity with given component
                cmd.entity(*entity)
                    .with_children(|cmd| match &item.equipment {
                        Some(equipment) => match equipment {
                            EquipmentType::Weapon(weapon) => {
                                cmd.spawn((
                                    weapon.clone(),
                                    item.clone(),
                                    Name::new(item.name.clone()),
                                    Transform::default_z(),
                                ));
                            }
                            EquipmentType::RepairBot(repair) => {
                                cmd.spawn((
                                    repair.clone(),
                                    item.clone(),
                                    Name::new(item.name.clone()),
                                    Transform::default_z(),
                                ));
                            }
                            EquipmentType::Energy(energy) => {
                                cmd.spawn((
                                    energy.clone(),
                                    item.clone(),
                                    Name::new(item.name.clone()),
                                    Transform::default_z(),
                                ));
                            }
                        },
                        None => unreachable!(),
                    });
            }
            events::EquipEvent::Unequip {
                entity,
                item,
                transfer_into_inventory: manage_inventory,
            } => {
                // get the inventory and equipment components of the given entity
                let (mut inventory, mut equipment) = inv_equip
                    .get_mut(*entity)
                    .map_err(|_| InventoryError::Unqueriable)?;

                // Find and remove the item entity from our children
                if let Some(item_entity) = children
                    .iter_descendants(*entity)
                    .find(|entity| items.get(*entity).map(|i| i == item).unwrap_or_default())
                {
                    cmd.entity(item_entity).despawn_recursive();

                    // shuffle inventory
                    if *manage_inventory {
                        equipment
                            .inventory
                            .transfer(item, &mut inventory, Some(1))?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// If entities are initialized with an equipment component instead of using events, we won't actually
/// add child entities. This system initializes equipment entities with child entities so we can enable this.
fn init_equipment(
    mut events: EventWriter<EquipEvent>,
    equipment: Query<(Entity, &Equipment), Added<Equipment>>,
) {
    for (entity, equip) in equipment.iter() {
        for (item, amount) in equip.inventory.iter() {
            for _ in 0..*amount {
                events.send(EquipEvent::Equip {
                    entity,
                    item: item.clone(),
                    transfer_from_inventory: false,
                });
            }
        }
    }
}

fn manage_drops(
    mut cmd: Commands,
    // Query for just-destroyed entities with a `Drop` component
    drops: Query<(&Drops, &Transform), Added<Destroyed>>,
) -> Result<(), InventoryError> {
    for (drops, transform) in drops.iter() {
        let mut inv = Inventory::default();
        // Filter with probabilities to find the items we will actually drop
        let mut rng = rand::thread_rng();

        let items_to_drop = drops.iter().filter_map(|(it, p)| {
            if rng.gen_ratio(1, p.d as u32) {
                Some((it, p))
            } else {
                None
            }
        });

        let mut rng = rand::thread_rng();
        for (item, amount) in items_to_drop {
            let amount = rng.gen_range(amount.min..=amount.max);
            inv.add(item.clone(), amount).unwrap();
        }

        // Spawn drops in a chest
        if !inv.is_empty() {
            cmd.spawn((
                Chest,
                inv,
                *transform,
                Collider::cuboid(0.5, 0.5, 0.5),
                CollisionLayers {
                    memberships: PhysicsCategory::Item.into(),
                    filters: LayerMask::NONE,
                },
            ));
        }
    }
    Ok(())
}

fn update_chests_in_range(
    mut chests_in_range: Query<(&Transform, &mut ChestsInRange)>,
    mut chests: Query<Entity, With<Chest>>,
    query: SpatialQuery,
) {
    for (transform, mut chest_in_range) in chests_in_range.iter_mut() {
        // Reset chests in range
        chest_in_range.chests.clear();
        // Cast a shape in our distance of reach and obtain all matching chest entities
        chest_in_range.chests = query
            .shape_intersections(
                &Collider::cylinder(chest_in_range.range, 1f32),
                transform.translation,
                Transform::default_z().rotation,
                SpatialQueryFilter {
                    mask: LayerMask::from(PhysicsCategory::Item),
                    excluded_entities: [].into(),
                },
            )
            .iter()
            // Get only chest entities
            .filter_map(|item| chests.get_mut(*item).ok())
            .collect();
    }
}

// #[cfg(test)]
// mod tests {
//     use events::{EquipEvent, InventoryEvent};

//     use super::*;

//     #[test]
//     fn create_with_capacity() {
//         // Setup app
//         let mut app = App::new();
//         app.add_plugins((InventoryPlugin,));
//     }
// }
