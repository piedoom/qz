use crate::prelude::*;
use avian3d::{
    collision::{Collider, LayerMask},
    prelude::{DistanceJoint, Joint},
    spatial_query::{SpatialQuery, SpatialQueryFilter},
};
use bevy::prelude::*;
use events::{DockEvent, StoreEvent};

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
                )
                    .run_if(in_state(AppState::main())),
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
                &SpatialQueryFilter {
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
                    if let Some(joint) = dockings
                        .get_mut(**dock)
                        .map(|mut docking| docking.remove(to_undock))
                        .ok()
                        .flatten()
                    {
                        cmd.entity(joint).despawn_recursive();
                    }
                }
            }
        }
    }
}

fn handle_store_events(
    mut events: EventReader<StoreEvent>,
    mut credits: Query<&mut Credits>,
    mut inventories: Query<&mut Inventory>,
    items: Res<Assets<Item>>,
) -> Result<(), StoreError> {
    for event in events.read() {
        match event {
            StoreEvent::Buy {
                buyer: patron,
                store: store_entity,
                item,
                quantity,
            }
            | StoreEvent::Sell {
                seller: patron,
                store: store_entity,
                item,
                quantity,
            } => {
                let is_buy_event = matches!(event, StoreEvent::Buy { .. });
                dbg!("a");
                let [patron_credits, store_credits] =
                    credits.get_many_mut([*patron, *store_entity])?;
                dbg!("b");
                let mut inventory = inventories.get_mut(*patron)?;
                let retrieved_item = items.get(item).unwrap();
                let (mut from_credits, mut to_credits) = {
                    match is_buy_event {
                        // Moving into the player inventory. ensure enough space
                        true => {
                            if inventory.space_remaining() >= retrieved_item.size * quantity {
                                (patron_credits, store_credits)
                            } else {
                                return Err(InventoryError::NoSpaceLeft {
                                    overage: (retrieved_item.size * quantity)
                                        - inventory.space_remaining(),
                                }
                                .into());
                            }
                        }
                        // Moving out of the player inventory. ensure items exist
                        false => {
                            if inventory.count(item) >= *quantity {
                                (store_credits, patron_credits)
                            } else {
                                return Err(StoreError::NotEnoughItems);
                            }
                        }
                    }
                };

                let Item { size, value, .. } = items
                    .get(item)
                    .expect("item should exist with given handle");
                let total_cost = value * quantity;
                if from_credits.get() < total_cost {
                    return Err(StoreError::NotEnoughCredits);
                }
                // Commit inventory and credit transfer
                from_credits.transfer(&mut to_credits, total_cost)?;
                if is_buy_event {
                    inventory.add(item.clone(), *size, *quantity)?;
                } else {
                    inventory.remove(item, *size, *quantity)?;
                }
            }
        }
    }
    Ok(())
}
