use crate::prelude::*;
use bevy::prelude::*;
use bevy_egui::*;
use events::EquipEvent;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_hud,));
    }
}

fn draw_hud(
    mut contexts: EguiContexts,
    mut events: EventWriter<EquipEvent>,
    inventory: Query<(Entity, &Inventory, &Equipment), With<Player>>,
) {
    egui::SidePanel::new(egui::panel::Side::Left, "hud").show(contexts.ctx_mut(), |ui| {
        for (entity, inventory, equipment) in inventory.iter() {
            ui.heading("Equipment");
            for (item, count) in equipment.inventory.items.iter() {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", item.name, count));
                        if ui.button("Unequip").clicked() {
                            events.send(EquipEvent::Unequip {
                                entity,
                                item: item.clone(),
                                manage_inventory: true,
                            });
                        }
                    });
                });
            }

            ui.heading("Inventory");

            for (item, count) in inventory.items.iter() {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", item.name, count));
                        if item.equipment.is_some() && ui.button("Equip").clicked() {
                            events.send(EquipEvent::Equip {
                                entity,
                                item: item.clone(),
                                manage_inventory: true,
                            });
                        }
                    });
                });
            }
        }
    });
}
