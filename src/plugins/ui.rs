use crate::prelude::*;
use bevy::prelude::*;
use bevy_egui::*;
use egui::Slider;
use events::{EquipEvent, InventoryEvent};

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
    inventories: Query<&Inventory>,
    energy: Query<&Energy>,
    children: Query<&Children>,
    player: Query<
        (
            Entity,
            &Inventory,
            &Equipment,
            &ChestsInRange,
            &Health,
            &Damage,
        ),
        With<Player>,
    >,
) {
    egui::SidePanel::new(egui::panel::Side::Left, "hud").show(contexts.ctx_mut(), |ui| {
        for (player_entity, player_inventory, equipment, chests_in_range, health, damage) in
            player.iter()
        {
            ui.heading("Status");
            ui.add(Slider::new(
                &mut (**health as f32 - **damage),
                0f32..=**health as f32,
            ));
            for child in children.iter_descendants(player_entity) {
                if let Ok(energy) = energy.get(child) {
                    ui.add(Slider::new(
                        &mut energy.charge.clone(),
                        0f32..=energy.capacity as f32,
                    ));
                }
            }

            ui.heading(format!(
                "{}/{}",
                player_inventory.space_occupied(),
                player_inventory.capacity()
            ));

            ui.separator();

            ui.heading("Equipment");
            for (item, count) in equipment.inventory.iter() {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", item.name, count));
                        if ui.button("Unequip").clicked() {
                            events.send(EquipEvent::Unequip {
                                entity: player_entity,
                                item: item.clone(),
                                transfer_into_inventory: true,
                            });
                        }
                        if ui.button("Inspect").clicked() {
                            *selected_item = Some(item.clone());
                        }
                    });
                });
            }

            ui.heading("Inventory");
            {
                for (item, count) in player_inventory.iter() {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} {}", item.name, count));
                            if item.equipment.is_some() && ui.button("Equip").clicked() {
                                events.send(EquipEvent::Equip {
                                    entity: player_entity,
                                    item: item.clone(),
                                    transfer_from_inventory: true,
                                });
                            }
                            if ui.button("Inspect").clicked() {
                                *selected_item = Some(item.clone());
                            }
                        });
                    });
                }
            }

            ui.separator();

            if !chests_in_range.chests.is_empty() {
                ui.heading("Chests");
                for chest in &chests_in_range.chests {
                    let chest_inventory = inventories.get(*chest).unwrap();
                    for (item, amount) in chest_inventory.iter() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} {}", item.name, amount));
                            if ui.button("Take").clicked() {
                                inv_events.send(InventoryEvent::Transfer {
                                    from: *chest,
                                    to: player_entity,
                                    item: item.clone(),
                                    amount: Some(*amount),
                                });
                            }
                            if ui.button("Inspect").clicked() {
                                *selected_item = Some(item.clone());
                            }
                        });
                    }
                }
            }
        }
    });
    if let Some(selected) = selected_item.as_ref() {
        egui::Window::new(&selected.name).show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                ui.label(format!("type: {}", selected.equipment_type_str(),));

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
                                ui.label(format!("tracking: {}", tracking));
                            }
                        },
                        EquipmentType::RepairBot(r) => {
                            ui.label(format!("repair rate: {}/s", r.rate));
                        }
                        EquipmentType::Energy(e) => {
                            ui.label(format!("capacity: {}", e.capacity));
                            ui.label(format!("recharge rate: {}/s", e.recharge_rate));
                        }
                    }
                }
            });
        });
    }
}
