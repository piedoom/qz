use crate::prelude::*;
use bevy::prelude::*;
use bevy_egui::*;
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
    inventories: Query<&Inventory>,
    player: Query<(Entity, &Inventory, &Equipment, &ChestsInRange), With<Player>>,
) {
    egui::SidePanel::new(egui::panel::Side::Left, "hud").show(contexts.ctx_mut(), |ui| {
        for (player_entity, player_inventory, equipment, chests_in_range) in player.iter() {
            ui.heading("Equipment");
            for (item, count) in equipment.inventory.items.iter() {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", item.name, count));
                        if ui.button("Unequip").clicked() {
                            events.send(EquipEvent::Unequip {
                                entity: player_entity,
                                item: item.clone(),
                                manage_inventory: true,
                            });
                        }
                    });
                });
            }

            ui.heading("Inventory");
            {
                for (item, count) in player_inventory.items.iter() {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} {}", item.name, count));
                            if item.equipment.is_some() && ui.button("Equip").clicked() {
                                events.send(EquipEvent::Equip {
                                    entity: player_entity,
                                    item: item.clone(),
                                    manage_inventory: true,
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
                    let chest_inventory = inventories.get(*chest).unwrap();
                    for (item, amount) in chest_inventory.items.iter() {
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
                        });
                    }
                }
            }
        }
    });
}
