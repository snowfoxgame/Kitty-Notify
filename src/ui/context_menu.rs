use crate::actions::execute_action;
use crate::config::{Config, Corner};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, GestureClick, Label, Orientation, Popover, Separator};
use std::cell::Cell;
use std::rc::Rc;

/// Attaches the right-click retro context menu to the pet cat sprite.
/// Strictly called in Desktop Pet Mode only.
pub fn attach_pet_context_menu(
    app: &Application,
    cat_sprite: &Box,
    corner: Corner,
    apply_input_mask: Rc<dyn Fn()>,
    win_weak: gtk4::glib::WeakRef<ApplicationWindow>,
) {
    let position = match corner {
        Corner::BottomRight | Corner::BottomLeft => gtk4::PositionType::Top,
        Corner::TopRight | Corner::TopLeft => gtk4::PositionType::Bottom,
    };

    let popover = Popover::builder()
        .has_arrow(true)
        .autohide(true)
        .position(position)
        .css_classes(["pixel-popover"])
        .build();
    popover.set_parent(cat_sprite);

    let menu_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();

    // 1. Settings button
    let btn_settings = Button::builder()
        .label("⚙ Settings")
        .css_classes(["pixel-menu-item"])
        .halign(gtk4::Align::Fill)
        .build();

    let app_settings = app.clone();
    let pop_settings = popover.clone();
    btn_settings.connect_clicked(move |_| {
        pop_settings.popdown();
        crate::ui::settings::open_settings_window(&app_settings);
    });
    menu_box.append(&btn_settings);

    // Separator 1
    let sep1 = Separator::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["pixel-menu-separator"])
        .build();
    menu_box.append(&sep1);

    // Dynamic custom actions container
    let actions_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .build();
    menu_box.append(&actions_box);

    // Separator 2
    let sep2 = Separator::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["pixel-menu-separator"])
        .build();
    menu_box.append(&sep2);

    // 2. Hide Pet / Close button
    let btn_hide = Button::builder()
        .label("⏹ Hide Pet")
        .css_classes(["pixel-menu-item", "pixel-menu-item-danger"])
        .halign(gtk4::Align::Fill)
        .build();

    let pop_hide = popover.clone();
    let win_weak_hide = win_weak.clone();
    btn_hide.connect_clicked(move |_| {
        pop_hide.popdown();
        crate::ipc::remove_pet_pid();
        let mut mem = crate::brain::CatMemory::load();
        mem.mark_clean_exit();
        if let Some(win) = win_weak_hide.upgrade() {
            win.close();
        }
    });
    menu_box.append(&btn_hide);

    popover.set_child(Some(&menu_box));

    // When popover closes, restore tight click-through input region
    let mask_on_close = Rc::clone(&apply_input_mask);
    let popover_active = Rc::new(Cell::new(false));
    let pa_close = Rc::clone(&popover_active);
    popover.connect_closed(move |_| {
        pa_close.set(false);
        mask_on_close();
    });

    // Secondary click (right-click) gesture
    let right_click = GestureClick::new();
    right_click.set_button(3); // Mouse button 3 = right-click

    let pop_trigger = popover.clone();
    let act_box_trigger = actions_box.clone();
    let win_weak_trigger = win_weak.clone();
    let pa_trigger = Rc::clone(&popover_active);

    right_click.connect_pressed(move |_, _, _, _| {
        // Dynamically populate custom actions from current config
        while let Some(child) = act_box_trigger.first_child() {
            act_box_trigger.remove(&child);
        }

        let cfg = Config::load();
        if cfg.custom_actions.is_empty() {
            let empty_label = Label::builder()
                .label("No actions configured")
                .css_classes(["pixel-menu-empty"])
                .margin_start(8)
                .margin_end(8)
                .margin_top(4)
                .margin_bottom(4)
                .build();
            act_box_trigger.append(&empty_label);
        } else {
            for act in cfg.custom_actions {
                let cmd = act.command.clone();
                let item_btn = Button::builder()
                    .label(format!("▶ {}", act.name))
                    .css_classes(["pixel-menu-item"])
                    .halign(gtk4::Align::Fill)
                    .build();

                let pop_item = pop_trigger.clone();
                item_btn.connect_clicked(move |_| {
                    pop_item.popdown();
                    execute_action(&cmd);
                });
                act_box_trigger.append(&item_btn);
            }
        }

        // Expand input region to allow full interaction with the floating popover
        if let Some(win) = win_weak_trigger.upgrade() {
            if let Some(surface) = win.surface() {
                surface.set_input_region(None);
            }
        }
        pa_trigger.set(true);
        pop_trigger.popup();
    });

    cat_sprite.add_controller(right_click);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_widget_creation() {
        if gtk4::init().is_err() {
            return;
        }
        let app = Application::builder().application_id("com.test.kittynotify.cm").build();
        let _ = app.register(gtk4::gio::Cancellable::NONE);
        let cat_box = Box::new(Orientation::Vertical, 0);
        let mask_called = Rc::new(Cell::new(false));
        let mask_cb = {
            let m = Rc::clone(&mask_called);
            Rc::new(move || m.set(true))
        };
        let win = ApplicationWindow::new(&app);
        attach_pet_context_menu(&app, &cat_box, Corner::BottomRight, mask_cb, win.downgrade());

        let mut found_gesture = false;
        let mut idx = 0;
        let controllers = cat_box.observe_controllers();
        while let Some(c) = controllers.item(idx) {
            if let Ok(gc) = c.downcast::<GestureClick>() {
                if gc.button() == 3 {
                    found_gesture = true;
                    break;
                }
            }
            idx += 1;
        }
        assert!(found_gesture, "Right-click GestureClick(3) should be attached to cat sprite");
    }
}

