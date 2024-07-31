//! Shields, basically
use bevy::prelude::*;
use events::EquipEvent;

use crate::prelude::*;

pub struct EquipmentPlugin;
impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_repairs,
                handle_energy,
                init_equipment,
                manage_equipment.pipe(handle_errors::<InventoryError>),
            ),
        );
    }
}

fn handle_repairs(
    mut damages: Query<(&mut Damage, &Children), Without<Destroyed>>,
    repairs: Query<&RepairBot>,
    time: Res<Time>,
) {
    for (mut damage, children) in damages.iter_mut() {
        if **damage != 0f32 {
            let repairs = children.iter().filter_map(|e| repairs.get(*e).ok());
            let repairs_rate = repairs.fold(0f32, |acc, i| acc + i.rate);
            let new_damage = (**damage - (repairs_rate * time.delta_seconds())).max(0f32); // It's OK if we go over damage as it'll just destroy the entity
            **damage = new_damage;
        }
    }
}

fn handle_energy(
    mut energies: Query<(&mut Energy, &Children), Without<Destroyed>>,
    generators: Query<&Generator>,
    batteries: Query<&Battery>,
    time: Res<Time>,
) {
    // Find base entities with energy components so we can loop through children (items)
    for (mut energy, children) in energies.iter_mut() {
        let generators = children.iter().filter_map(|e| generators.get(*e).ok());
        let batteries = children.iter().filter_map(|e| batteries.get(*e).ok());

        // Find our maximum energy. If no batteries exist, we'll use the generator per second instead
        let recharge_rate = generators.fold(0f32, |acc, i| acc + i.recharge_rate);
        let mut max_energy = batteries.fold(0f32, |acc, i| acc + i.capacity());
        if max_energy == 0f32 {
            // ugly imperative code :(
            max_energy = recharge_rate;
        }

        // Add the charge to our `Energy` component
        *energy += (recharge_rate * time.delta_seconds()).into();
        energy.clamp(max_energy);
    }
}

fn manage_equipment(
    mut cmd: Commands,
    mut events: EventReader<events::EquipEvent>,
    mut inv_equip: Query<(&mut Inventory, &mut Equipment)>,
    children: Query<&Children>,
    items: Query<&Handle<Item>>,
    item_assets: Res<Assets<Item>>,
) -> Result<(), InventoryError> {
    for event in events.read() {
        match event {
            events::EquipEvent::Equip {
                entity: parent_entity,
                item,
                transfer_from_inventory,
            } => {
                // get the inventory and equipment components of the given entity
                let (mut inventory, mut equipment) = inv_equip
                    .get_mut(*parent_entity)
                    .map_err(|_| InventoryError::Unqueriable)?;

                if *transfer_from_inventory {
                    // shuffle inventory
                    inventory.transfer(item.clone(), &mut equipment.inventory, 1, &item_assets)?;
                }

                // Retrieve item
                let retrieved_item = item_assets.get(item).ok_or(InventoryError::ItemNotFound)?;

                // Add entity with given component
                // Note we need to do this manually or `on_add` will not trigger
                match &retrieved_item.equipment {
                    Some(equipment) => {
                        let entity = cmd
                            .spawn((
                                item.clone(),
                                Name::new(retrieved_item.name.clone()),
                                TransformBundle::default_z(),
                            ))
                            .id();

                        // Manually set up parent/child relationship
                        cmd.entity(*parent_entity).add_child(entity);
                        cmd.entity(entity).set_parent(*parent_entity);

                        let mut entity = cmd.entity(entity);
                        match equipment {
                            EquipmentType::Weapon(weapon) => {
                                entity.insert(weapon.clone());
                            }
                            EquipmentType::RepairBot(repair) => {
                                entity.insert(repair.clone());
                            }
                            EquipmentType::Generator(generator) => {
                                entity.insert(generator.clone());
                            }
                            EquipmentType::Battery(battery) => {
                                entity.insert(battery.clone());
                            }
                            EquipmentType::Armor(armor) => {
                                entity.insert(armor.clone());
                            }
                        };
                    }
                    None => unreachable!(),
                };
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

                    // transfer equipment inventory into regular inventory
                    if *manage_inventory {
                        equipment.inventory.transfer(
                            item.clone(),
                            &mut inventory,
                            1,
                            &item_assets,
                        )?;
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
