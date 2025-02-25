use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.observe(on_transfer.pipe(handle_errors::<InventoryError>))
            .observe(on_toss_overboard.pipe(handle_errors::<InventoryError>))
            .add_systems(
                Update,
                (
                    (
                        recalculate_inventory_equipment_mass,
                        manage_drops.pipe(handle_errors::<InventoryError>),
                    )
                        .run_if(resource_exists::<Library>),
                    update_chests_in_range.run_if(resource_exists::<SpatialQueryPipeline>),
                ),
            );
    }
}

fn on_transfer(
    trigger: Trigger<triggers::InventoryTransfer>,
    mut inventories: Query<&mut Inventory>,
    items: Res<Assets<Item>>,
) -> Result<(), InventoryError> {
    let triggers::InventoryTransfer { from, to, transfer } = trigger.event();
    let [mut from, mut to] = inventories
        .get_many_mut([*from, *to])
        .map_err(|_| InventoryError::Unqueriable)?;

    match transfer {
        triggers::InventoryTransferSettings::Item { item, quantity } => {
            from.transfer(item.clone(), &mut to, *quantity, &items)?;
        }
        triggers::InventoryTransferSettings::All => {
            from.transfer_all(&mut to)?;
        }
    }
    Ok(())
}

fn on_toss_overboard(
    trigger: Trigger<triggers::TossItemOverboard>,
    mut cmd: Commands,
    mut inventories: Query<&mut Inventory>,
    items: Res<Assets<Item>>,
    transforms: Query<&Transform>,
) -> Result<(), InventoryError> {
    let triggers::TossItemOverboard {
        entity,
        item,
        quantity,
    } = trigger.event();
    let mut inventory = inventories.get_mut(*entity)?;
    let retrieved_item = items.get(item).ok_or(InventoryError::ItemNotFound)?;
    let transform = transforms.get(*entity)?;

    inventory.remove(item, retrieved_item.size, *quantity)?;

    // Spawn tossed stuff in a chest
    cmd.spawn((
        Chest,
        Inventory::max_capacity().with(item.clone(), *quantity, &items)?,
        *transform,
        Collider::cuboid(0.5, 0.5, 0.5),
        CollisionLayers {
            memberships: PhysicsCategory::Item.into(),
            filters: LayerMask::NONE,
        },
    ));
    Ok(())
}

fn manage_drops(
    mut cmd: Commands,
    // Query for just-destroyed entities with a `Drop` component
    drops: Query<(&Drops, Option<&Credits>, &Transform), Added<Destroyed>>,
    library: Res<Library>,
    items: Res<Assets<Item>>,
) -> Result<(), InventoryError> {
    for (drops, maybe_credits, transform) in drops.iter() {
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
                TransformBundle::from_transform(*transform),
                Collider::cuboid(0.5, 0.5, 0.5),
                CollisionLayers {
                    memberships: PhysicsCategory::Item.into(),
                    filters: LayerMask::NONE,
                },
                RigidBody::Dynamic,
                AngularVelocity(Vec3::new(
                    rng.gen::<f32>() - 0.5f32,
                    rng.gen::<f32>() - 0.5f32,
                    rng.gen::<f32>() - 0.5f32,
                )),
                LinearVelocity(Vec3::new(
                    rng.gen::<f32>() - 0.5f32,
                    rng.gen::<f32>() - 0.5f32,
                    0f32,
                )),
                Model::new(library.model("items/chest").unwrap()).with_offset(-Vec3::Y * 2f32),
            ));
        }

        // Spawn credits
        if let Some(credits) = maybe_credits {
            cmd.spawn((
                TransformBundle::from_transform(*transform),
                Chest,
                *credits,
                Collider::cuboid(0.5, 0.5, 0.5),
                CollisionLayers {
                    memberships: PhysicsCategory::Item.into(),
                    filters: LayerMask::NONE,
                },
                LinearVelocity(Vec3::new(
                    rng.gen::<f32>() - 0.5f32,
                    rng.gen::<f32>() - 0.5f32,
                    0f32,
                )),
                Model::new(library.model("items/credits").unwrap()).with_offset(-Vec3::Y * 2f32),
            ));
        }
    }
    Ok(())
}

// Recalculate the craft mass when it is changed
fn recalculate_inventory_equipment_mass(
    mut query: Query<
        (&mut Mass, Option<&Inventory>, Option<&Equipped>, &Craft),
        Or<(Changed<Inventory>, Changed<Equipped>)>,
    >,
    item_assets: Res<Assets<Item>>,
    items: Query<&Equipment>,
) {
    for (mut mass, inventory, equipped, craft) in query.iter_mut() {
        **mass = craft.mass;
        if let Some(inventory) = inventory {
            **mass += inventory.mass(&item_assets);
        }
        if let Some(equipped) = equipped {
            **mass += equipped.mass(&item_assets, &items);
        }
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
