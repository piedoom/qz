use crate::prelude::*;
use bevy::prelude::*;
use events::EquipEvent;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EquipEvent>().add_systems(
            Update,
            (manage_equipment.pipe(handle_errors), init_equipment),
        );
    }
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
                manage_inventory,
            } => {
                // get the inventory and equipment components of the given entity
                let (mut inventory, mut equipment) = inv_equip
                    .get_mut(*entity)
                    .map_err(|_| InventoryError::Unqueriable)?;

                if *manage_inventory {
                    // shuffle inventory
                    inventory.move_to(item, &mut equipment.inventory, Some(1))?;
                }

                // Add entity with given component
                cmd.entity(*entity)
                    .with_children(|cmd| match &item.equipment {
                        Some(equipment) => match equipment {
                            EquipmentType::Weapon(weapon) => {
                                cmd.spawn((weapon.clone(), item.clone(), Name::new(item.name)));
                            }
                        },
                        None => unreachable!(),
                    });
            }
            events::EquipEvent::Unequip {
                entity,
                item,
                manage_inventory,
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
                        equipment.inventory.move_to(item, &mut inventory, Some(1))?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_errors(In(result): In<Result<(), InventoryError>>) {
    if let Err(e) = result {
        eprintln!("{e}");
    }
}

/// If entities are initialized with an equipment component instead of using events, we won't actually
/// add child entities. This system initializes equipment entities with child entities so we can enable this.
fn init_equipment(
    mut events: EventWriter<EquipEvent>,
    equipment: Query<(Entity, &Equipment), Added<Equipment>>,
) {
    for (entity, equip) in equipment.iter() {
        for (item, amount) in &equip.inventory.items {
            for _ in 0..*amount {
                events.send(EquipEvent::Equip {
                    entity,
                    item: item.clone(),
                    manage_inventory: false,
                });
            }
        }
    }
}
