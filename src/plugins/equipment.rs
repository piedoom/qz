//! Shields, basically
use bevy::prelude::*;

use crate::prelude::*;

/// Handles equipment running logic
pub struct EquipmentPlugin;
impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_repairs,
                handle_energy,
                manage_overheating,
                manage_equipped_builders.run_if(resource_exists::<Library>),
            )
                .run_if(in_state(AppState::main())),
        )
        .observe(on_equip.pipe(handle_errors::<EquipmentError>))
        .observe(on_unequip.pipe(handle_errors::<EquipmentError>));
    }
}

/// Repair damage for entities with a [`RepairBot`] equipped
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

/// Generate energy and recharge batteries
///
/// # System overview
///
/// 1. Get all (parent) entities that have [`Energy`]
/// 2. Find all generator and battery children (TODO: change to new equips tracking system)
/// 3. Charge batteries (parent energy) with the rate specified in the generator (Note that batteries actually just determine
///     total potential energy storage, which is what the [`Energy`] component on the parent tracks)
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

fn on_equip(
    trigger: Trigger<triggers::Equip>,
    mut cmd: Commands,
    mut inventories: Query<&mut Inventory>,
    items: Res<Assets<Item>>,
    equipped: Query<&Equipped>,
) -> Result<(), EquipmentError> {
    let triggers::Equip {
        entity: parent_entity,
        item,
        transfer_from_inventory,
    } = trigger.event();

    // get the inventory and equipped components of the given entity
    let equipped = equipped.get(*parent_entity)?;

    // Test that there is a slot available
    let retrieved_item = items.get(item).unwrap().clone();
    let id = retrieved_item.equipment.unwrap().id();
    if equipped.slots_remaining(&id) != 0 {
        if *transfer_from_inventory {
            // Remove item from inventory
            let mut inventory = inventories.get_mut(*parent_entity)?;
            inventory.remove(item, retrieved_item.size, 1)?;
        }
        // Add equip as a child
        let equipment_entity = cmd.spawn(()).id();
        cmd.entity(*parent_entity).add_child(equipment_entity);
        cmd.entity(equipment_entity)
            .set_parent(*parent_entity)
            .insert(Equipment::new(item.clone()));
    } else {
        return Err(EquipmentError::SlotNotAvailable);
    }
    Ok(())
}

fn on_unequip(
    trigger: Trigger<triggers::Unequip>,
    mut cmd: Commands,
    mut inventories: Query<&mut Inventory>,
    equipments: Query<&Equipment>,
    items: Res<Assets<Item>>,
    parents: Query<&Parent>,
) -> Result<(), EquipmentError> {
    let triggers::Unequip {
        equipment,
        transfer_into_inventory,
    } = trigger.event();
    if *transfer_into_inventory {
        let eq = equipments.get(*equipment)?;
        let retrieved_item = items
            .get(&eq.handle())
            .ok_or(InventoryError::ItemNotFound)?;
        // get parent entity
        let entity = parents.get(*equipment)?.get();
        let mut inventory = inventories.get_mut(entity)?;
        inventory.add(eq.handle(), retrieved_item.size, 1)?;
    }
    cmd.entity(*equipment).despawn_recursive();
    Ok(())
}

fn manage_equipped_builders(
    mut cmd: Commands,
    library: Res<Library>,
    builders: Query<(Entity, &EquippedBuilder)>,
) {
    // Add necessary equipped and child entities
    for (parent_entity, builder) in builders.iter() {
        // Add empty equipped
        cmd.entity(parent_entity).try_insert((Equipped {
            equipped: [].into(),
            slots: builder.slots.iter().cloned().collect(),
        },));

        // Add children (manually)
        // Child hooks will automatically update the equipment slots,
        // but we do assume the `EquippedBuilder` defines a valid
        // configuration
        for item in &builder.equipped {
            // Spawn the equipment entity
            let equipment_entity = cmd.spawn(()).id();
            // get the item from the string
            let item_handle = library.item(item).clone().unwrap();
            // Manually set up parent/child relationship. If we use the child builder,
            // it won't work, see: https://github.com/bevyengine/bevy/issues/14545
            cmd.entity(parent_entity).add_child(equipment_entity);
            cmd.entity(equipment_entity).set_parent(parent_entity);
            cmd.entity(equipment_entity)
                .insert(Equipment::new(item_handle.clone()));
        }

        cmd.entity(parent_entity).remove::<EquippedBuilder>();
    }
}

fn manage_overheating(
    mut cmd: Commands,
    heats: Query<(Entity, &Heat, Option<&Overheated>), Changed<Heat>>,
) {
    for (equipment_entity, heat, maybe_overheated) in heats.iter() {
        if heat.get() == 1f32 {
            // Overheated, add the component
            cmd.entity(equipment_entity).insert(Overheated);
        } else if heat.get() == 0f32 && maybe_overheated.is_some() {
            // No longer overheated. Remove the overheated component
            cmd.entity(equipment_entity).remove::<Overheated>();
        }
    }
}
