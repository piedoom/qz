use crate::prelude::*;
use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32},
    EguiContexts,
};

pub(super) fn draw_ui(
    mut contexts: EguiContexts,
    mut events: EventWriter<events::EquipEvent>,
    mut inv_events: EventWriter<events::InventoryEvent>,
    mut selected_item: Local<Option<Item>>,
    mut store_events: EventWriter<events::StoreEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    equipment: Query<&Equipment>,
    items: Res<Assets<Item>>,
    inventories: Query<&Inventory>,
    batteries: Query<&Battery>,
    children: Query<&Children>,
    stores: Query<&Store>,
    player: Query<
        (
            Entity,
            &Energy,
            &Inventory,
            &Equipped,
            &ChestsInRange,
            &Health,
            &Damage,
            Option<&Docked>,
            &Credits,
        ),
        With<Player>,
    >,
) {
    if let Ok((
        player_entity,
        energy,
        player_inventory,
        equipped,
        chests_in_range,
        health,
        damage,
        maybe_docked,
        credits,
    )) = player.get_single()
    {
        // allocate space near the bottom of the screen
        let size = egui::Vec2::new(500f32, 100f32);
        egui::Area::new("game_ui".into())
            .anchor(Align2::CENTER_BOTTOM, egui::Vec2::new(0f32, 24f32))
            .default_size(size)
            .show(contexts.ctx_mut(), |ui| {
                ui.horizontal(|ui| {
                    let bar_size = (12f32, size.y);

                    // Health bar
                    ui.add(widgets::Bar {
                        size: bar_size.into(),
                        range: 0f32..=health.get() as f32,
                        value: health.get() as f32 - **damage,
                        vertical: true,
                        ..Default::default()
                    });

                    // Energy
                    // TODO: use hooks for this instead of calculating every time
                    let player_batteries = children
                        .iter_descendants(player_entity)
                        .filter_map(|child| batteries.get(child).ok());
                    let capacity = player_batteries.fold(0f32, |acc, i| acc + i.capacity());

                    // Show equipped items
                    ui.horizontal(|ui| {
                        // Each equipment type that is equipped gets a section
                        // Completely empty slot types are hidden
                        // let mut equipped_grouped = equipped.iter().collect::<Vec<_>>();
                        // for equipment_groups in equipped_grouped
                        //     .equipped_grouped
                        //     .chunk_by(|(a, _), (b, _)| a == b)
                        // {}
                        for equipment_group in equipped.iter() {
                            // TODO
                        }
                    });

                    ui.add(widgets::Bar {
                        size: bar_size.into(),
                        range: 0f32..=capacity,
                        value: energy.charge(),
                        fill: Color32::BLUE,
                        vertical: true,
                        ..Default::default()
                    });
                });
            });

        egui::SidePanel::new(egui::panel::Side::Left, "ui").show(contexts.ctx_mut(), |ui| {
            for (
                player_entity,
                _energy,
                player_inventory,
                equipped,
                chests_in_range,
                _health,
                _damage,
                maybe_docked,
                credits,
            ) in player.iter()
            {
                if ui.button("save").clicked() {
                    next_state.set(AppState::SaveGame);
                }

                ui.heading(format!("Credits: {}", credits.get()));

                ui.separator();

                ui.heading("Equipment");
                // ui.heading(format!(
                //     "Equipment ({}/{})",
                //     equipment.inventory.space_occupied(),
                //     equipment.inventory.capacity()
                // ));
                for chunk in equipped
                    .equipped
                    .iter()
                    .collect::<Vec<_>>()
                    .chunk_by(|a, b| a.0 == b.0)
                {
                    for (equipment_id, entities) in chunk.iter() {
                        ui.vertical(|ui| {
                            ui.heading(equipment_id.to_string());
                            for equipment_entity in entities.iter() {
                                let handle = equipment.get(*equipment_entity).unwrap().handle();
                                let retrieved_item = items
                                    .get(&handle)
                                    .ok_or(InventoryError::ItemNotFound)
                                    .unwrap();
                                ui.horizontal(|ui| {
                                    ui.label(retrieved_item.name.to_string());

                                    if ui.button("Inspect").clicked() {
                                        *selected_item = Some(retrieved_item.clone());
                                    }

                                    if ui.button("Unequip").clicked() {
                                        events.send(events::EquipEvent::Unequip {
                                            entity: player_entity,
                                            equipment: *equipment_entity,
                                            transfer_into_inventory: true,
                                        });
                                    }
                                });
                            }
                        });
                    }
                }

                ui.heading(format!(
                    "Inventory ({}/{})",
                    player_inventory.space_occupied(),
                    player_inventory.capacity()
                ));
                {
                    for (item, count) in player_inventory.iter() {
                        let retrieved_item =
                            items.get(item).ok_or(InventoryError::ItemNotFound).unwrap();
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("{} {}", retrieved_item.name, count));

                                if ui.button("Toss").clicked() {
                                    inv_events.send(events::InventoryEvent::TossOverboard {
                                        entity: player_entity,
                                        item: item.clone(),
                                        amount: 1,
                                    });
                                }

                                if ui.button("Inspect").clicked() {
                                    *selected_item = Some(retrieved_item.clone());
                                }

                                if retrieved_item.equipment.is_some()
                                    && ui.button("Equip").clicked()
                                {
                                    events.send(events::EquipEvent::Equip {
                                        entity: player_entity,
                                        item: item.clone(),
                                        transfer_from_inventory: true,
                                    });
                                }
                            });
                        });
                    }
                }

                ui.separator();

                if !chests_in_range.chests.is_empty() {
                    ui.heading("Chests");
                    for chest in &chests_in_range.chests {
                        if let Ok(chest_inventory) = inventories.get(*chest) {
                            for (item, amount) in chest_inventory.iter() {
                                let retrieved_item =
                                    items.get(item).ok_or(InventoryError::ItemNotFound).unwrap();
                                ui.horizontal(|ui| {
                                    ui.label(format!("{} {}", retrieved_item.name, amount));
                                    if ui.button("Take").clicked() {
                                        inv_events.send(events::InventoryEvent::Transfer {
                                            from: *chest,
                                            to: player_entity,
                                            item: item.clone(),
                                            amount: *amount,
                                        });
                                    }
                                    if ui.button("Inspect").clicked() {
                                        *selected_item = Some(retrieved_item.clone());
                                    }
                                });
                            }
                        }
                    }
                }

                if let Some(docked) = maybe_docked.as_ref() {
                    egui::Window::new("docked").show(ui.ctx(), |ui| {
                        ui.vertical(|ui| {
                            if let Ok(store) = stores.get(***docked) {
                                for item in store.items.iter() {
                                    let retrieved_item = items.get(item).unwrap();
                                    ui.horizontal(|ui| {
                                        ui.label(retrieved_item.name.to_string());
                                        // ui.label(

                                        //     options
                                        //         .buy
                                        //         .value(retrieved_item.value)
                                        //         .map(|x| x.to_string())
                                        //         .unwrap_or("None".to_string()),
                                        // );
                                        ui.label(retrieved_item.value.to_string());

                                        if ui.button("buy 1").clicked() {
                                            store_events.send(events::StoreEvent::Buy {
                                                buyer: player_entity,
                                                store: ***docked,
                                                item: item.clone(),
                                                quantity: 1,
                                            });
                                        }

                                        // if let Some(buying_for) =
                                        //     options.buy.value(retrieved_item.value)
                                        // {
                                        //     if ui.button("sell 1").clicked() {
                                        //         store_events.send(StoreEvent::Sell {
                                        //             seller: player_entity,
                                        //             store: ***docked,
                                        //             item: item.clone(),
                                        //             quantity: 1,
                                        //             price: buying_for,
                                        //         });
                                        //     }
                                        // }
                                    });
                                }
                            }
                        });
                    });
                }
            }
        });
        if let Some(selected) = selected_item.as_ref() {
            egui::Window::new(&selected.name).show(contexts.ctx_mut(), |ui| {
                ui.vertical(|ui| {
                    ui.label(format!("type: {}", selected.equipment_type_str(),));
                    ui.label(format!("value: {}", selected.value));
                    ui.label(format!("size: {}", selected.size));
                    ui.label(format!("mass: {}", selected.mass));

                    if let Some(eq) = &selected.equipment {
                        match eq {
                            EquipmentType::Weapon(w) => match &w.weapon_type {
                                WeaponType::ProjectileWeapon {
                                    tracking,
                                    speed,
                                    recoil,
                                    spread,
                                    shots,
                                    damage,
                                    radius,
                                    lifetime,
                                    energy,
                                    projectile_model: _,
                                    distance,
                                } => {
                                    ui.heading("projectile weapon");
                                    ui.label(format!("damage: {}", damage));
                                    ui.label(format!("energy: {}", energy));
                                    ui.label(format!("recoil: {}s", recoil));
                                    ui.label(format!("shots: {}", shots));
                                    ui.label(format!("spread: {}", spread));
                                    ui.label(format!("distance: {}", distance));
                                    ui.label(format!("lifetime: {}", lifetime));
                                    ui.label(format!("speed: {}", speed));
                                    ui.label(format!("size: {}", radius * 2f32));
                                    ui.label(format!("tracking: {}", tracking));
                                }
                                WeaponType::LaserWeapon {
                                    tracking,
                                    damage_per_second,
                                    energy_per_second,
                                    range,
                                    width,
                                    activation_energy,
                                    color: _color,
                                    heat_per_second,
                                    cooling_per_second,
                                } => {
                                    ui.heading("laser");
                                    ui.label(format!("range: {range}"));
                                    ui.label(format!("damage: {damage_per_second}/s"));
                                    ui.label(format!("energy: {energy_per_second}/s"));
                                    ui.label(format!("width: {width}"));
                                    ui.label(format!("tracking: {tracking}"));
                                    ui.label(format!("activation energy: {activation_energy}"));
                                    ui.label(format!(
                                        "heat generated: {:2}%/s",
                                        heat_per_second * 10.0
                                    ));
                                    ui.label(format!(
                                        "cooling: {:2}%/s",
                                        cooling_per_second * 10.0
                                    ));
                                }
                            },
                            EquipmentType::RepairBot(r) => {
                                ui.label(format!("repair rate: {}/s", r.rate));
                            }
                            EquipmentType::Generator(e) => {
                                ui.label(format!("recharge rate: {}/s", e.recharge_rate));
                            }
                            EquipmentType::Battery(b) => {
                                ui.label(format!("capacity: {}", b.capacity()));
                            }
                            EquipmentType::Armor(a) => {
                                ui.label(format!("armor: {}", a.health));
                            }
                        }
                    }
                });
            });
        }
    }
}

pub(super) fn draw_minimaps(
    mut contexts: EguiContexts,
    query: Query<(&'static GlobalTransform, &'static Collider)>,
    gate_query: Query<(&'static GlobalTransform, &'static Gate)>,
    player_transform: Query<&Transform, With<Player>>,
    universe: Res<Universe>,
    maybe_universe_position: Option<Res<UniversePosition>>,
) {
    let _node: Vec<_> = universe.graph.node_weights().collect();

    if let Ok(player_transform) = player_transform.get_single() {
        let translation = player_transform.translation.truncate();
        egui::Area::new("minimap".into())
            .interactable(false)
            .anchor(Align2::RIGHT_TOP, (0f32, 0f32))
            .show(contexts.ctx_mut(), |ui| {
                ui.add(widgets::ZoneMap {
                    size: egui::Vec2::new(240f32, 240f32),
                    scale: 4f32,
                    collider_query: &query,
                    world_center: (translation.x, translation.y).into(),
                    gate_query: &gate_query,
                    universe_graph: &universe.graph,
                });

                ui.add(widgets::UniverseMap {
                    size: egui::Vec2::new(240f32, 480f32),
                    graph: Some(&universe.graph),
                    current_position: maybe_universe_position,
                });
            });
    }
}
