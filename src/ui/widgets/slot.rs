use crate::prelude::*;

use bevy_egui::egui::*;

/// Health bar, shields, etc.
pub struct Slot<'a> {
    /// Equipment
    pub equipment_type_id: EquipmentTypeId,
    /// Size of the slot container
    pub size: Vec2,
    /// Each equipment item and its corresponding entity
    pub equipped: Vec<(&'a Item, bevy::prelude::Entity)>,
    pub capacity: usize,
}

impl<'a> Widget for Slot<'a> {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> Response {
        let Self {
            equipment_type_id,
            size,
            equipped,
            capacity,
        } = self;
        let (mut rect, response) = ui.allocate_at_least(size, Sense::hover());
        {
            let painter = ui.painter();

            // Paint the widget background
            painter.rect_filled(rect, 0f32, Color32::DARK_GRAY);

            painter.text(
                rect.center_top(),
                Align2::CENTER_TOP,
                equipment_type_id,
                FontId::proportional(16f32),
                Color32::GREEN,
            );
        }

        // adjust the rect to give the top title thing some room
        rect.set_top(rect.top() + 24f32);

        // Can't zip because we need zip_longest
        for i in 0..capacity {
            let maybe_equipped = equipped.get(i);
            // for (i, (item, entity)) in equipped.iter().enumerate() {
            // Obtain the rect space relative to our currently drawn equipment
            let item_rect = {
                // The height for each item in this slot is determined by the equipped capacity for that item ID
                let item_height = rect.height() / capacity as f32;
                // Create a new rect with the given height and offset its top by `i`
                let mut item_rect = rect;
                item_rect.set_top(rect.top() + (i as f32 * item_height));
                item_rect.set_bottom(rect.top() + ((i + 1) as f32 * item_height));
                item_rect
            };

            let item_response = ui.allocate_rect(item_rect, Sense::click());
            let painter = ui.painter_at(item_rect);

            match maybe_equipped {
                Some((item, entity)) => {
                    painter.rect_stroke(item_rect, 0f32, Stroke::new(2f32, Color32::GREEN));
                    painter.text(
                        item_rect.left_center(),
                        Align2::LEFT_CENTER,
                        item.name.clone(),
                        FontId::proportional(16f32),
                        Color32::GREEN,
                    );
                    match item.equipment.as_ref() {
                        Some(EquipmentType::Weapon(weapon)) => match &weapon.weapon_type {
                            WeaponType::ProjectileWeapon {
                                tracking,
                                speed,
                                recoil,
                                spread,
                                shots,
                                damage,
                                radius,
                                lifetime,
                                distance,
                                energy,
                                projectile_model,
                            } => {}
                            WeaponType::LaserWeapon {
                                tracking,
                                damage_per_second,
                                energy_per_second,
                                range,
                                width,
                                activation_energy,
                                heat_per_second,
                                cooling_per_second,
                                color,
                            } => {
                                // ui.add(Bar {
                                //     size: (item_rect.width(), 3f32).into(),
                                //     range: weapon.,
                                //     value: todo!(),
                                //     radius: todo!(),
                                //     fill: todo!(),
                                //     ..Default::default()
                                // });
                            }
                        },
                        _ => (),
                    }
                }
                None => {
                    painter.rect_stroke(item_rect, 0f32, Stroke::new(2f32, Color32::GRAY));
                    painter.text(
                        item_rect.left_center(),
                        Align2::LEFT_CENTER,
                        "Empty",
                        FontId::proportional(16f32),
                        Color32::GRAY,
                    );
                }
            }
        }

        response
    }
}
