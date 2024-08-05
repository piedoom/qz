use crate::prelude::*;
use bevy::prelude::*;
use bevy_egui::*;
use egui::{Align2, Color32, Slider, Stroke};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use events::{EquipEvent, InventoryEvent, StoreEvent};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_hud,));
    }
}

fn draw_hud(
    mut contexts: EguiContexts,
    mut events: EventWriter<EquipEvent>,
    mut inv_events: EventWriter<InventoryEvent>,
    mut selected_item: Local<Option<Item>>,
    mut store_events: EventWriter<StoreEvent>,
    mut errors: EventReader<GameError>,
    maybe_universe_position: Option<Res<UniversePosition>>,
    universe: Res<Universe>,
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
    egui::SidePanel::new(egui::panel::Side::Left, "hud").show(contexts.ctx_mut(), |ui| {
        for (
            player_entity,
            energy,
            player_inventory,
            equipped,
            chests_in_range,
            health,
            damage,
            maybe_docked,
            credits,
        ) in player.iter()
        {
            ui.heading("Status");
            ui.add(Slider::new(
                &mut (**health as f32 - **damage),
                0f32..=**health as f32,
            ));

            let player_batteries = children
                .iter_descendants(player_entity)
                .filter_map(|child| batteries.get(child).ok());
            let capacity = player_batteries.fold(0f32, |acc, i| acc + i.capacity());

            ui.add(Slider::new(&mut energy.charge().clone(), 0f32..=capacity));
            // for child in children.iter_descendants(player_entity) {
            //     if let Ok(energy) = energy.get(child) {
            //         ui.add(Slider::new(
            //             &mut energy.charge().clone(),
            //             0f32..=energy.capacity as f32,
            //         ));
            //     }
            // }

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
                                    events.send(EquipEvent::Unequip {
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
                                inv_events.send(InventoryEvent::TossOverboard {
                                    entity: player_entity,
                                    item: item.clone(),
                                    amount: 1,
                                });
                            }

                            if ui.button("Inspect").clicked() {
                                *selected_item = Some(retrieved_item.clone());
                            }

                            if retrieved_item.equipment.is_some() && ui.button("Equip").clicked() {
                                events.send(EquipEvent::Equip {
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
                                    inv_events.send(InventoryEvent::Transfer {
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
                            for (item, options) in store.items.iter() {
                                let retrieved_item = items.get(item).unwrap();
                                ui.horizontal(|ui| {
                                    ui.label(retrieved_item.name.to_string());

                                    ui.label(
                                        options
                                            .buy
                                            .value(retrieved_item.value)
                                            .map(|x| x.to_string())
                                            .unwrap_or("None".to_string()),
                                    );
                                    ui.label(
                                        options
                                            .sell
                                            .value(retrieved_item.value)
                                            .map(|x| x.to_string())
                                            .unwrap_or("None".to_string()),
                                    );

                                    if let Some(selling_for) =
                                        options.sell.value(retrieved_item.value)
                                    {
                                        if ui.button("buy 1").clicked() {
                                            store_events.send(StoreEvent::Buy {
                                                buyer: player_entity,
                                                store: ***docked,
                                                item: item.clone(),
                                                quantity: 1,
                                                price: selling_for,
                                            });
                                        }
                                    }

                                    if let Some(buying_for) =
                                        options.buy.value(retrieved_item.value)
                                    {
                                        if ui.button("sell 1").clicked() {
                                            store_events.send(StoreEvent::Sell {
                                                seller: player_entity,
                                                store: ***docked,
                                                item: item.clone(),
                                                quantity: 1,
                                                price: buying_for,
                                            });
                                        }
                                    }
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
                        EquipmentType::Weapon(w) => match w.weapon_type {
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
                            } => {
                                ui.heading("projectile weapon");
                                ui.label(format!("damage: {}", damage));
                                ui.label(format!("energy: {}", energy));
                                ui.label(format!("recoil: {}s", recoil));
                                ui.label(format!("shots: {}", shots));
                                ui.label(format!("spread: {}", spread));
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
                            } => {
                                ui.heading("laser");
                                ui.label(format!("range: {}s", range));
                                ui.label(format!("damage: {}/s", damage_per_second));
                                ui.label(format!("energy: {}/s", energy_per_second));
                                ui.label(format!("width: {}s", width));
                                ui.label(format!("tracking: {}", tracking));
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

    let mut toasts = Toasts::new()
        .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0)) // 10 units from the bottom right corner
        .direction(egui::Direction::BottomUp);

    for error in errors.read() {
        toasts.add(Toast {
            text: format!("{error}").into(),
            kind: ToastKind::Error,
            options: ToastOptions::default()
                .duration_in_seconds(5.0)
                .show_progress(true),
            ..Default::default()
        });
    }

    egui::CentralPanel::default()
        .frame(egui::Frame {
            fill: Color32::TRANSPARENT,
            stroke: Stroke::NONE,
            ..default()
        })
        .show(contexts.ctx_mut(), |ui| {
            if let Some(universe_position) = maybe_universe_position {
                if let Some(node) = universe.graph.node_weight(universe_position.get()) {
                    ui.horizontal_top(|ui| {
                        ui.heading(node.name.clone());
                        ui.heading(format!("Depth: {}", node.depth.clone()));
                    });
                }
            }
            toasts.show(ui.ctx());
        });
}
