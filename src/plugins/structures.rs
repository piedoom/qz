use crate::prelude::*;
use avian3d::{
    collision::{Collider, LayerMask},
    prelude::{DistanceJoint, Joint},
    spatial_query::{SpatialQuery, SpatialQueryFilter},
};
use bevy::{ecs::query::QueryEntityError, prelude::*};
use events::{DockEvent, StoreEvent};
use thiserror::Error;

pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DockEvent>()
            .add_event::<StoreEvent>()
            .add_systems(
                Update,
                (
                    update_dock_in_ranges,
                    update_dockings,
                    handle_store_events.pipe(handle_errors::<StoreError>),
                ),
            );
    }
}

fn update_dock_in_ranges(
    mut dock_in_ranges: Query<(&Transform, &mut DockInRange)>,
    mut docks: Query<Entity, With<Dockings>>,
    query: SpatialQuery,
) {
    for (dock_transform, mut dock_in_range) in dock_in_ranges.iter_mut() {
        // Reset
        dock_in_range.dock = None;
        // Cast a shape in our distance of reach and obtain the first dockable entity
        dock_in_range.dock = query
            .shape_intersections(
                &Collider::cylinder(dock_in_range.range, 1f32),
                dock_transform.translation,
                Transform::default_z().rotation,
                SpatialQueryFilter {
                    mask: LayerMask::from(PhysicsCategory::Structure),
                    excluded_entities: [].into(),
                },
            )
            .iter()
            // Get only dockable entities
            .filter_map(|structure| docks.get_mut(*structure).ok())
            .next();
    }
}

fn update_dockings(
    mut cmd: Commands,
    mut events: EventReader<DockEvent>,
    mut dockings: Query<&mut Dockings>,
    docked: Query<&Docked>,
) {
    for event in events.read() {
        match event {
            DockEvent::Dock { to_dock, dock } => {
                cmd.entity(*to_dock).insert(Docked(*dock));
                let joint = cmd
                    .spawn((DistanceJoint::new(*to_dock, *dock)
                        .with_limits(0.0, 3.0)
                        .with_compliance(0.05),))
                    .id();
                dockings.get_mut(*dock).unwrap().insert(*to_dock, joint);
            }
            DockEvent::Undock { to_undock } => {
                if let Ok(dock) = docked.get(*to_undock) {
                    cmd.entity(*to_undock).remove::<Docked>();
                    if let Some(joint) = dockings.get_mut(**dock).unwrap().remove(to_undock) {
                        cmd.entity(joint).despawn_recursive();
                    }
                }
            }
        }
    }
}

fn handle_store_events(
    mut events: EventReader<StoreEvent>,
    mut stores: Query<(&mut Inventory, &mut Credits), With<Store>>,
    mut inventories: Query<(&mut Inventory, &mut Credits), Without<Store>>,
    items: Res<Assets<Item>>,
) -> Result<(), StoreError> {
    for event in events.read() {
        match event {
            StoreEvent::Buy {
                buyer: patron,
                store: store_entity,
                item,
                quantity,
                price,
            }
            | StoreEvent::Sell {
                seller: patron,
                store: store_entity,
                item,
                quantity,
                price,
            } => {
                let (store_inventory, store_credits) = stores.get_mut(*store_entity)?;
                let (inventory, credits) = inventories.get_mut(*patron)?;
                let (mut from_inventory, mut to_inventory, mut from_credits, mut to_credits) = {
                    match matches!(event, StoreEvent::Buy { .. }) {
                        true => (store_inventory, inventory, credits, store_credits),
                        false => (inventory, store_inventory, store_credits, credits),
                    }
                };

                let retrieved_item = items
                    .get(item)
                    .expect("item should exist with given handle");

                // Ensure there is enough space left
                if to_inventory.space_remaining() >= retrieved_item.size * quantity {
                    // Ensure the inventory has the item available for transfer of the specified quantity
                    if from_inventory.quantity(&item) < *quantity {
                        return Err(StoreError::NotEnoughItems);
                    }
                    // Ensure there is enough credits to transfer
                    let total_cost = price * quantity;
                    match from_credits.get() >= total_cost {
                        true => {
                            // Commit inventory and credit transfer
                            from_credits.transfer(&mut to_credits, total_cost)?;
                            from_inventory.transfer(
                                item.clone(),
                                &mut to_inventory,
                                *quantity,
                                &items,
                            )?;
                        }
                        false => return Err(StoreError::NotEnoughCredits),
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
enum StoreError {
    #[error(transparent)]
    QueryEntityError(#[from] QueryEntityError),
    #[error(transparent)]
    CreditsError(#[from] CreditsError),
    #[error(transparent)]
    InventoryError(#[from] InventoryError),
    #[error("not enough items")]
    NotEnoughItems,
    #[error("not enough credits")]
    NotEnoughCredits,
}
