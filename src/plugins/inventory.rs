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
                    recalculate_inventory_equipment_mass,
                    manage_inventory.pipe(handle_errors::<InventoryError>),
                    manage_drops.pipe(handle_errors::<InventoryError>),
                    update_chests_in_range.run_if(resource_exists::<SpatialQueryPipeline>),
                ),
            );
    }
}

fn manage_inventory(
    mut cmd: Commands,
    mut inventories: Query<&mut Inventory>,
    mut events: EventReader<events::InventoryEvent>,
    transforms: Query<&Transform>,
    items: Res<Assets<Item>>,
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
                from.transfer(item.clone(), &mut to, *amount, &items)?;
            }
            InventoryEvent::TransferAll { from, to } => {
                let [mut from, mut to] = inventories
                    .get_many_mut([*from, *to])
                    .map_err(|_| InventoryError::Unqueriable)?;
                from.transfer_all(&mut to)?;
            }
            InventoryEvent::TossOverboard {
                entity,
                item,
                amount,
            } => {
                let mut inventory = inventories.get_mut(*entity)?;
                let retrieved_item = items.get(item).ok_or(InventoryError::ItemNotFound)?;
                let transform = transforms.get(*entity)?;

                inventory.remove(item.clone(), retrieved_item.size, *amount)?;

                // Spawn tossed stuff in a chest
                cmd.spawn((
                    Chest,
                    Inventory::max_capacity().with(item.clone(), *amount, &items)?,
                    *transform,
                    Collider::cuboid(0.5, 0.5, 0.5),
                    CollisionLayers {
                        memberships: PhysicsCategory::Item.into(),
                        filters: LayerMask::NONE,
                    },
                ));
            }
        }
    }
    Ok(())
}

fn manage_drops(
    mut cmd: Commands,
    // Query for just-destroyed entities with a `Drop` component
    drops: Query<(&Drops, &Transform), Added<Destroyed>>,
    items: Res<Assets<Item>>,
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
            let retrieved_item = items.get(item).ok_or(InventoryError::ItemNotFound)?;
            inv.add(item.clone(), retrieved_item.size, amount).unwrap();
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

// Recalculate the craft mass when it is changed
fn recalculate_inventory_equipment_mass(
    mut query: Query<
        (&mut Mass, &Inventory, &Equipped, &Craft),
        Or<(Changed<Inventory>, Changed<Equipped>)>,
    >,
    item_assets: Res<Assets<Item>>,
    items: Query<&Equipment>,
) {
    for (mut mass, inventory, equipped, craft) in query.iter_mut() {
        **mass = inventory.mass(&item_assets) + equipped.mass(&item_assets, &items) + craft.mass;
    }
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
