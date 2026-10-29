use crate::config::{AnimationSpeed, CatSkin, Config, Corner, DisplayMode, Personality, SoundEffect};
use crate::ui::bubble::{build_animated_cat_sprite_with_speed, build_speech_bubble};
use crate::ui::layer_shell::show_popups;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, CheckButton, ColorButton, DropDown, Entry, Label,
    Orientation, Scale, ScrolledWindow, StringList,
};
use std::cell::Cell;
use std::process::Command;
use std::rc::Rc;

fn rgba_to_hex(rgba: &gtk4::gdk::RGBA) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (rgba.red() * 255.0).round().clamp(0.0, 255.0) as u8,
        (rgba.green() * 255.0).round().clamp(0.0, 255.0) as u8,
        (rgba.blue() * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

fn update_preview_cat(box_widget: &Box, skin: CatSkin, scale: f32, speed_multiplier: f32) {
    while let Some(child) = box_widget.first_child() {
        box_widget.remove(&child);
    }
    let speed_cell = Rc::new(Cell::new(speed_multiplier));
    let cat = build_animated_cat_sprite_with_speed(skin, scale, Some(speed_cell), None);
    box_widget.append(&cat);
}

pub fn open_settings_window(app: &Application) {
    let config = Config::load();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("KittyNotify Settings")
        .default_width(480)
        .default_height(520)
        .resizable(true)
        .css_classes(["settings-window"])
        .build();

    // Enforce minimum window bounds so UI shrinks cleanly to 420x400
    window.set_size_request(420, 400);

    // Dynamic CSS Provider for settings background & subtle repeating 8-bit paw pattern
    let settings_css_provider = gtk4::CssProvider::new();
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &settings_css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER + 20,
        );
    }
    settings_css_provider.load_from_data(&crate::assets::generate_settings_background_css(
        &config.visual.bubble_bg_color,
    ));

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(14)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    // --- Header Card ---
    let header_card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .css_classes(["settings-header-card"])
        .build();

    let title = Label::builder()
        .label("🐾 KittyNotify Settings")
        .css_classes(["settings-title"])
        .xalign(0.0)
        .build();
    let subtitle = Label::builder()
        .label("Pixel-art desktop update notifier for Arch Linux & KDE Plasma")
        .css_classes(["settings-subtitle"])
        .xalign(0.0)
        .build();
    header_card.append(&title);
    header_card.append(&subtitle);
    main_box.append(&header_card);

    // --- State trackers for Appearance & Live Preview ---
    let current_bg = Rc::new(std::cell::RefCell::new(config.visual.bubble_bg_color.clone()));
    let current_text = Rc::new(std::cell::RefCell::new(config.visual.bubble_text_color.clone()));
    let current_skin = Rc::new(Cell::new(config.visual.cat_skin));
    let current_scale = Rc::new(Cell::new(config.visual.cat_scale));
    let current_speed = Rc::new(Cell::new(config.visual.animation_speed));

    // --- 1. Live Preview & Appearance Customizer Card ---
    let card_appearance = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["settings-card"])
        .build();

    let sec_preview = Label::builder()
        .label("APPEARANCE & LIVE PREVIEW")
        .css_classes(["settings-group-header"])
        .xalign(0.0)
        .build();
    card_appearance.append(&sec_preview);

    // Live Preview Box
    let preview_container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::End)
        .css_classes(["pixel-preview-container"])
        .build();

    let preview_bubble = build_speech_bubble(
        5,
        0,
        0,
        Corner::BottomRight,
        &config.visual.bubble_bg_color,
        &config.visual.bubble_text_color,
        || {},
    );
    preview_bubble.set_custom_message("Arch has 5 updates ready!");
    preview_bubble.widget.set_valign(gtk4::Align::End);

    let preview_cat_box = Box::builder()
        .orientation(Orientation::Vertical)
        .valign(gtk4::Align::End)
        .build();
    update_preview_cat(
        &preview_cat_box,
        config.visual.cat_skin,
        config.visual.cat_scale,
        config.visual.animation_speed.multiplier(),
    );

    preview_container.append(&preview_bubble.widget);
    preview_container.append(&preview_cat_box);
    card_appearance.append(&preview_container);

    // Cat Skin Dropdown
    let skin_label = Label::builder().label("Cat Skin:").xalign(0.0).build();
    card_appearance.append(&skin_label);

    let skins = ["Carton Cat (Box)", "Dracula Cat (Vampire)", "Classic Idle Cat"];
    let skin_list = StringList::new(&skins);
    let skin_dropdown = DropDown::builder().model(&skin_list).build();
    skin_dropdown.set_selected(match config.visual.cat_skin {
        CatSkin::CartonCat => 0,
        CatSkin::DraculaCat => 1,
        CatSkin::IdleCat => 2,
    });
    card_appearance.append(&skin_dropdown);

    // Animation Speed Dropdown
    let speed_label = Label::builder().label("Animation Speed:").xalign(0.0).build();
    card_appearance.append(&speed_label);

    let speeds = ["0.5x (Slow)", "1.0x (Default)", "2.0x (Fast)"];
    let speed_list = StringList::new(&speeds);
    let speed_dropdown = DropDown::builder().model(&speed_list).build();
    speed_dropdown.set_selected(match config.visual.animation_speed {
        AnimationSpeed::Half => 0,
        AnimationSpeed::Double => 2,
        _ => 1,
    });
    card_appearance.append(&speed_dropdown);

    // Cat Scale Slider
    let scale_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let scale_title = Label::builder().label("Cat Sprite Scale:").xalign(0.0).hexpand(true).build();
    let scale_val_label = Label::builder()
        .label(format!("{:.2}x", config.visual.cat_scale))
        .build();
    scale_box.append(&scale_title);
    scale_box.append(&scale_val_label);
    card_appearance.append(&scale_box);

    let cat_scale_slider = Scale::with_range(Orientation::Horizontal, 1.0, 3.0, 0.1);
    cat_scale_slider.set_value(config.visual.cat_scale as f64);
    cat_scale_slider.set_increments(0.1, 0.25);
    cat_scale_slider.set_digits(2);
    cat_scale_slider.set_draw_value(false);
    card_appearance.append(&cat_scale_slider);

    // Speech Bubble Colors (Background & Text)
    let colors_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .homogeneous(true)
        .build();

    let bg_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    let bg_label = Label::builder().label("Bubble Background:").xalign(0.0).build();
    let bg_color_btn = ColorButton::new();
    bg_color_btn.set_use_alpha(false);
    let bg_rgba = gtk4::gdk::RGBA::parse(&config.visual.bubble_bg_color)
        .unwrap_or(gtk4::gdk::RGBA::WHITE);
    bg_color_btn.set_rgba(&bg_rgba);
    bg_box.append(&bg_label);
    bg_box.append(&bg_color_btn);

    let text_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    let text_label = Label::builder().label("Bubble Text Color:").xalign(0.0).build();
    let text_color_btn = ColorButton::new();
    text_color_btn.set_use_alpha(false);
    let text_rgba = gtk4::gdk::RGBA::parse(&config.visual.bubble_text_color)
        .unwrap_or(gtk4::gdk::RGBA::BLACK);
    text_color_btn.set_rgba(&text_rgba);
    text_box.append(&text_label);
    text_box.append(&text_color_btn);

    colors_row.append(&bg_box);
    colors_row.append(&text_box);
    card_appearance.append(&colors_row);

    main_box.append(&card_appearance);

    // --- Dynamic Live Preview Connections ---
    let cur_skin_for_dd = Rc::clone(&current_skin);
    let cur_scale_for_dd = Rc::clone(&current_scale);
    let cur_speed_for_skin = Rc::clone(&current_speed);
    let preview_cat_for_skin = preview_cat_box.clone();
    skin_dropdown.connect_selected_notify(move |dd| {
        let skin = match dd.selected() {
            1 => CatSkin::DraculaCat,
            2 => CatSkin::IdleCat,
            _ => CatSkin::CartonCat,
        };
        cur_skin_for_dd.set(skin);
        update_preview_cat(
            &preview_cat_for_skin,
            skin,
            cur_scale_for_dd.get(),
            cur_speed_for_skin.get().multiplier(),
        );
    });

    let cur_skin_for_scale = Rc::clone(&current_skin);
    let cur_scale_for_slider = Rc::clone(&current_scale);
    let cur_speed_for_scale = Rc::clone(&current_speed);
    let preview_cat_for_scale = preview_cat_box.clone();
    let scale_lbl_clone = scale_val_label.clone();
    cat_scale_slider.connect_value_changed(move |scale_w| {
        let val = scale_w.value() as f32;
        cur_scale_for_slider.set(val);
        scale_lbl_clone.set_label(&format!("{:.2}x", val));
        update_preview_cat(
            &preview_cat_for_scale,
            cur_skin_for_scale.get(),
            val,
            cur_speed_for_scale.get().multiplier(),
        );
    });

    let cur_skin_for_speed = Rc::clone(&current_skin);
    let cur_scale_for_speed = Rc::clone(&current_scale);
    let cur_speed_for_dd = Rc::clone(&current_speed);
    let preview_cat_for_speed = preview_cat_box.clone();
    speed_dropdown.connect_selected_notify(move |dd| {
        let speed = match dd.selected() {
            0 => AnimationSpeed::Half,
            2 => AnimationSpeed::Double,
            _ => AnimationSpeed::Normal,
        };
        cur_speed_for_dd.set(speed);
        update_preview_cat(
            &preview_cat_for_speed,
            cur_skin_for_speed.get(),
            cur_scale_for_speed.get(),
            speed.multiplier(),
        );
        let _ = crate::ipc::try_send_ipc_command(&format!("speed {}", speed.multiplier()));
    });

    let cur_bg_for_btn = Rc::clone(&current_bg);
    let cur_txt_for_bg_btn = Rc::clone(&current_text);
    let bubble_for_bg = preview_bubble.clone();
    let settings_css_provider_for_bg = settings_css_provider.clone();
    bg_color_btn.connect_color_set(move |btn| {
        let hex = rgba_to_hex(&btn.rgba());
        *cur_bg_for_btn.borrow_mut() = hex.clone();
        bubble_for_bg.set_colors(&hex, &cur_txt_for_bg_btn.borrow());
        settings_css_provider_for_bg.load_from_data(
            &crate::assets::generate_settings_background_css(&hex),
        );
    });

    let cur_bg_for_txt_btn = Rc::clone(&current_bg);
    let cur_txt_for_btn = Rc::clone(&current_text);
    let bubble_for_txt = preview_bubble.clone();
    text_color_btn.connect_color_set(move |btn| {
        let hex = rgba_to_hex(&btn.rgba());
        *cur_txt_for_btn.borrow_mut() = hex.clone();
        bubble_for_txt.set_colors(&cur_bg_for_txt_btn.borrow(), &hex);
    });

    // --- 2. Display & Positioning Card ---
    let card_display = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["settings-card"])
        .build();

    let sec_display = Label::builder()
        .label("DISPLAY & POSITIONING")
        .css_classes(["settings-group-header"])
        .xalign(0.0)
        .build();
    card_display.append(&sec_display);

    // Screen Corner
    let corner_label = Label::builder().label("Screen Corner:").xalign(0.0).build();
    card_display.append(&corner_label);

    let corners = ["Bottom-Right", "Bottom-Left", "Top-Right", "Top-Left"];
    let corner_list = StringList::new(&corners);
    let corner_dropdown = DropDown::builder().model(&corner_list).build();
    corner_dropdown.set_selected(match config.general.corner {
        Corner::BottomRight => 0,
        Corner::BottomLeft => 1,
        Corner::TopRight => 2,
        Corner::TopLeft => 3,
    });
    card_display.append(&corner_dropdown);

    // Target Monitor Dropdown
    let mon_label = Label::builder().label("Target Monitor:").xalign(0.0).build();
    card_display.append(&mon_label);

    let mon_options = if let Some(display) = gtk4::gdk::Display::default() {
        crate::ui::monitor::get_available_monitor_options(&display)
    } else {
        vec![crate::ui::monitor::MonitorOption {
            id: "auto".to_string(),
            label: "Primary Display (Auto)".to_string(),
        }]
    };

    let mon_labels: Vec<&str> = mon_options.iter().map(|o| o.label.as_str()).collect();
    let mon_list = StringList::new(&mon_labels);
    let mon_dropdown = DropDown::builder().model(&mon_list).build();

    let mut selected_mon_idx = 0;
    for (idx, opt) in mon_options.iter().enumerate() {
        if opt.id.eq_ignore_ascii_case(&config.general.monitor) {
            selected_mon_idx = idx as u32;
            break;
        }
    }
    mon_dropdown.set_selected(selected_mon_idx);
    card_display.append(&mon_dropdown);

    // Display Mode
    let mode_label = Label::builder().label("Display Mode (Layer Shell):").xalign(0.0).build();
    card_display.append(&mode_label);

    let modes = ["Above All Windows (Overlay)", "Desktop Only (Behind Apps)"];
    let mode_list = StringList::new(&modes);
    let mode_dropdown = DropDown::builder().model(&mode_list).build();
    mode_dropdown.set_selected(match config.general.display_mode {
        DisplayMode::Overlay => 0,
        DisplayMode::DesktopOnly => 1,
    });
    card_display.append(&mode_dropdown);

    // Desktop Pet Mode Toggle
    let pet_mode_check = CheckButton::builder()
        .label("Always Visible (Desktop Pet Mode)")
        .active(config.visual.always_visible_pet_mode)
        .build();
    card_display.append(&pet_mode_check);

    // Multi-monitor Checkbox
    let multi_mon_check = CheckButton::builder()
        .label("Show on all connected monitors simultaneously")
        .active(config.general.show_on_all_monitors)
        .build();
    card_display.append(&multi_mon_check);

    let mon_dd_sens = mon_dropdown.clone();
    mon_dd_sens.set_sensitive(!config.general.show_on_all_monitors);
    multi_mon_check.connect_toggled(move |btn| {
        mon_dd_sens.set_sensitive(!btn.is_active());
    });

    main_box.append(&card_display);

    // --- 3. Desktop Pet Banter Card ---
    let card_banter = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["settings-card"])
        .build();

    let sec_banter = Label::builder()
        .label("DESKTOP PET BANTER")
        .css_classes(["settings-group-header"])
        .xalign(0.0)
        .build();
    card_banter.append(&sec_banter);

    let banter_check = CheckButton::builder()
        .label("Enable Autonomous Banter (Random Quips)")
        .active(config.banter.enabled)
        .build();
    card_banter.append(&banter_check);

    let banter_sound_check = CheckButton::builder()
        .label("Play Quiet Sound on Banter")
        .active(config.banter.play_sound)
        .build();
    card_banter.append(&banter_sound_check);

    // Personality Dropdown
    let personality_label = Label::builder().label("Cat Personality:").xalign(0.0).build();
    card_banter.append(&personality_label);

    let personalities = [
        "Normal (Balanced Mix)",
        "Cute (Sweet & Wholesome)",
        "Sassy (Teasing & Roasts)",
        "Lazy (Sleepy & Reluctant)",
    ];
    let personality_list = StringList::new(&personalities);
    let personality_dropdown = DropDown::builder().model(&personality_list).build();
    personality_dropdown.set_selected(match config.banter.personality {
        Personality::Normal => 0,
        Personality::Cute => 1,
        Personality::Sassy => 2,
        Personality::Lazy => 3,
    });
    card_banter.append(&personality_dropdown);

    // Min interval
    let min_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let min_title = Label::builder().label("Min Interval:").xalign(0.0).hexpand(true).build();
    let min_val_label = Label::builder()
        .label(format!("{}s", config.banter.interval_min))
        .build();
    min_box.append(&min_title);
    min_box.append(&min_val_label);
    card_banter.append(&min_box);

    let min_scale = Scale::with_range(Orientation::Horizontal, 15.0, 300.0, 5.0);
    min_scale.set_value(config.banter.interval_min as f64);
    min_scale.set_digits(0);
    min_scale.set_draw_value(false);
    let min_lbl_clone = min_val_label.clone();
    min_scale.connect_value_changed(move |scale| {
        let val = scale.value().round() as u32;
        min_lbl_clone.set_label(&format!("{}s", val));
    });
    card_banter.append(&min_scale);

    // Max interval
    let max_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let max_title = Label::builder().label("Max Interval:").xalign(0.0).hexpand(true).build();
    let max_val_label = Label::builder()
        .label(format!("{}s", config.banter.interval_max))
        .build();
    max_box.append(&max_title);
    max_box.append(&max_val_label);
    card_banter.append(&max_box);

    let max_scale = Scale::with_range(Orientation::Horizontal, 30.0, 600.0, 10.0);
    max_scale.set_value(config.banter.interval_max as f64);
    max_scale.set_digits(0);
    max_scale.set_draw_value(false);
    let max_lbl_clone = max_val_label.clone();
    max_scale.connect_value_changed(move |scale| {
        let val = scale.value().round() as u32;
        max_lbl_clone.set_label(&format!("{}s", val));
    });
    card_banter.append(&max_scale);

    let b_sound_sens = banter_sound_check.clone();
    let b_pers_sens = personality_dropdown.clone();
    let b_min_sens = min_scale.clone();
    let b_max_sens = max_scale.clone();
    banter_check.connect_toggled(move |btn| {
        let active = btn.is_active();
        b_sound_sens.set_sensitive(active);
        b_pers_sens.set_sensitive(active);
        b_min_sens.set_sensitive(active);
        b_max_sens.set_sensitive(active);
    });
    banter_sound_check.set_sensitive(config.banter.enabled);
    personality_dropdown.set_sensitive(config.banter.enabled);
    min_scale.set_sensitive(config.banter.enabled);
    max_scale.set_sensitive(config.banter.enabled);

    main_box.append(&card_banter);

    // --- 4. Audio Feedback Card ---
    let card_audio = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["settings-card"])
        .build();

    let sec_audio = Label::builder()
        .label("AUDIO FEEDBACK")
        .css_classes(["settings-group-header"])
        .xalign(0.0)
        .build();
    card_audio.append(&sec_audio);

    let audio_check = CheckButton::builder()
        .label("Enable Audio Feedback")
        .active(config.audio.enabled)
        .build();
    card_audio.append(&audio_check);

    let sound_label = Label::builder().label("Sound Effect:").xalign(0.0).build();
    card_audio.append(&sound_label);

    let sounds = ["Realistic Cat Meow", "8-Bit Chiptune", "Mute"];
    let sound_list = StringList::new(&sounds);
    let sound_dropdown = DropDown::builder().model(&sound_list).build();
    sound_dropdown.set_selected(match config.audio.sound_effect {
        SoundEffect::Realistic => 0,
        SoundEffect::Chiptune => 1,
        SoundEffect::Mute => 2,
    });
    card_audio.append(&sound_dropdown);

    let volume_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let volume_title = Label::builder().label("Sound Volume:").xalign(0.0).hexpand(true).build();
    let volume_val_label = Label::builder()
        .label(format!("{}%", config.audio.sound_volume))
        .build();
    volume_box.append(&volume_title);
    volume_box.append(&volume_val_label);
    card_audio.append(&volume_box);

    let volume_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 5.0);
    volume_scale.set_value(config.audio.sound_volume as f64);
    volume_scale.set_digits(0);
    volume_scale.set_draw_value(false);

    let val_label_clone = volume_val_label.clone();
    volume_scale.connect_value_changed(move |scale| {
        let val = scale.value().round() as u32;
        val_label_clone.set_label(&format!("{}%", val));
    });
    card_audio.append(&volume_scale);

    let sound_dd_sens = sound_dropdown.clone();
    let volume_scale_sens = volume_scale.clone();
    audio_check.connect_toggled(move |btn| {
        let active = btn.is_active();
        sound_dd_sens.set_sensitive(active);
        volume_scale_sens.set_sensitive(active);
    });
    sound_dropdown.set_sensitive(config.audio.enabled);
    volume_scale.set_sensitive(config.audio.enabled);

    main_box.append(&card_audio);

    // --- 4. Package Filtering Card ---
    let card_pkg = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["settings-card"])
        .build();

    let sec_pkg = Label::builder()
        .label("PACKAGE FILTERING")
        .css_classes(["settings-group-header"])
        .xalign(0.0)
        .build();
    card_pkg.append(&sec_pkg);

    let ignore_label = Label::builder()
        .label("Ignored Packages (comma-separated):")
        .xalign(0.0)
        .build();
    card_pkg.append(&ignore_label);

    let ignore_entry = Entry::builder()
        .text(config.updates.ignore_packages.join(", "))
        .build();
    card_pkg.append(&ignore_entry);

    main_box.append(&card_pkg);

    // --- 5. Smart Helper & Behavioral Memory Card ---
    let card_smart = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["settings-card"])
        .build();

    let sec_smart = Label::builder()
        .label("SMART HELPER & BEHAVIORAL MEMORY")
        .css_classes(["settings-group-header"])
        .xalign(0.0)
        .build();
    card_smart.append(&sec_smart);

    let smart_check = CheckButton::builder()
        .label("Enable Smart Helper & Behavioral Memory")
        .active(config.smart.enabled)
        .build();
    card_smart.append(&smart_check);

    let smart_sub_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_start(16)
        .build();

    let cache_alert_check = CheckButton::builder()
        .label(format!(
            "Pacman Cache Alert (> {:.0} GB)",
            config.smart.pacman_cache_limit_gb
        ))
        .active(config.smart.pacman_cache_alert)
        .build();
    smart_sub_box.append(&cache_alert_check);

    let orphan_alert_check = CheckButton::builder()
        .label(format!(
            "Orphan Packages Alert (> {} orphans)",
            config.smart.orphan_limit
        ))
        .active(config.smart.orphan_alert)
        .build();
    smart_sub_box.append(&orphan_alert_check);

    let failed_services_check = CheckButton::builder()
        .label("Failed Systemd Services Alert")
        .active(config.smart.failed_services_alert)
        .build();
    smart_sub_box.append(&failed_services_check);

    let btn_reset_memory = Button::builder()
        .label("🔄 Reset Cat Memory")
        .tooltip_text("Wipes memory.json, resetting dismiss streaks and habit tracking")
        .css_classes(["pixel-button"])
        .margin_top(4)
        .build();

    let btn_rm_clone = btn_reset_memory.clone();
    btn_reset_memory.connect_clicked(move |_| {
        let _ = crate::brain::CatMemory::reset();
        btn_rm_clone.set_label("✔ Cat Memory Reset!");
    });
    smart_sub_box.append(&btn_reset_memory);

    let ssb_clone = smart_sub_box.clone();
    smart_check.connect_toggled(move |btn| {
        ssb_clone.set_sensitive(btn.is_active());
    });
    smart_sub_box.set_sensitive(config.smart.enabled);

    card_smart.append(&smart_sub_box);
    main_box.append(&card_smart);

    // --- 6. Right-Click Quick Actions (Desktop Pet) Card ---
    let card_quick_actions = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["settings-card"])
        .build();

    let sec_qa = Label::builder()
        .label("RIGHT-CLICK QUICK ACTIONS (DESKTOP PET)")
        .css_classes(["settings-group-header"])
        .xalign(0.0)
        .build();
    card_quick_actions.append(&sec_qa);

    let custom_actions_rc = Rc::new(std::cell::RefCell::new(config.custom_actions.clone()));
    let list_container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    type RenderHolder = Rc<std::cell::RefCell<Option<std::boxed::Box<dyn Fn()>>>>;
    let render_holder: RenderHolder = Rc::new(std::cell::RefCell::new(None));

    let rh_init = Rc::clone(&render_holder);
    let list_c_init = list_container.clone();
    let actions_init = Rc::clone(&custom_actions_rc);

    *rh_init.borrow_mut() = Some(std::boxed::Box::new(move || {
        while let Some(child) = list_c_init.first_child() {
            list_c_init.remove(&child);
        }

        let acts = actions_init.borrow();
        if acts.is_empty() {
            let empty_lbl = Label::builder()
                .label("No quick actions configured. Add one below!")
                .css_classes(["pixel-menu-empty"])
                .xalign(0.0)
                .build();
            list_c_init.append(&empty_lbl);
        } else {
            for (idx, act) in acts.iter().enumerate() {
                let row = Box::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(8)
                    .css_classes(["action-item-row"])
                    .build();

                let text_box = Box::builder()
                    .orientation(Orientation::Vertical)
                    .spacing(2)
                    .hexpand(true)
                    .build();

                let name_lbl = Label::builder()
                    .label(&act.name)
                    .css_classes(["action-item-name"])
                    .xalign(0.0)
                    .build();
                let cmd_lbl = Label::builder()
                    .label(&act.command)
                    .css_classes(["action-item-cmd"])
                    .xalign(0.0)
                    .build();
                text_box.append(&name_lbl);
                text_box.append(&cmd_lbl);
                row.append(&text_box);

                let btn_del = Button::builder()
                    .label("Delete")
                    .css_classes(["pixel-button", "pixel-button-danger"])
                    .valign(gtk4::Align::Center)
                    .build();

                let act_del_rc = Rc::clone(&actions_init);
                let rh_del = Rc::clone(&render_holder);
                btn_del.connect_clicked(move |_| {
                    let mut a = act_del_rc.borrow_mut();
                    if idx < a.len() {
                        a.remove(idx);
                        let mut cfg = Config::load();
                        cfg.custom_actions = a.clone();
                        let _ = cfg.save();
                    }
                    drop(a);
                    if let Some(r) = rh_del.borrow().as_ref() {
                        r();
                    }
                });
                row.append(&btn_del);
                list_c_init.append(&row);
            }
        }
    }));

    if let Some(r) = rh_init.borrow().as_ref() {
        r();
    }
    card_quick_actions.append(&list_container);

    // Prominent Security Disclaimer Callout Box (visible right above the command inputs)
    let disclaimer_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["disclaimer-banner"])
        .margin_top(8)
        .margin_bottom(2)
        .build();

    let disc_icon = Label::builder()
        .label("⚠️")
        .css_classes(["disclaimer-icon"])
        .valign(gtk4::Align::Start)
        .build();
    let disc_text = Label::builder()
        .label("Notice: Executing commands with sudo or elevated root privileges is strongly discouraged. KittyNotify is an experimental pet tool and takes no responsibility or liability for system damage, misconfigurations, or data loss caused by elevated commands. Run commands at your own risk.")
        .css_classes(["disclaimer-text"])
        .wrap(true)
        .xalign(0.0)
        .build();
    disclaimer_box.append(&disc_icon);
    disclaimer_box.append(&disc_text);
    card_quick_actions.append(&disclaimer_box);

    // "Add Action" Form
    let add_form_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(4)
        .build();

    let entry_name = Entry::builder()
        .placeholder_text("Action Name (e.g. Work Setup)")
        .build();
    let entry_cmd = Entry::builder()
        .placeholder_text("Command to run (e.g. firefox & kitty)")
        .build();

    let btn_add_action = Button::builder()
        .label("+ Add Quick Action")
        .css_classes(["pixel-button", "pixel-button-primary"])
        .hexpand(true)
        .build();

    let en_add = entry_name.clone();
    let ec_add = entry_cmd.clone();
    let act_add_rc = Rc::clone(&custom_actions_rc);
    let rh_add = Rc::clone(&rh_init);

    btn_add_action.connect_clicked(move |_| {
        let name = en_add.text().trim().to_string();
        let cmd = ec_add.text().trim().to_string();
        if !name.is_empty() && !cmd.is_empty() {
            let mut a = act_add_rc.borrow_mut();
            a.push(crate::config::CustomAction {
                name,
                command: cmd,
            });
            let mut cfg = Config::load();
            cfg.custom_actions = a.clone();
            let _ = cfg.save();
            drop(a);
            en_add.set_text("");
            ec_add.set_text("");
            if let Some(r) = rh_add.borrow().as_ref() {
                r();
            }
        }
    });

    add_form_box.append(&entry_name);
    add_form_box.append(&entry_cmd);
    add_form_box.append(&btn_add_action);
    card_quick_actions.append(&add_form_box);

    main_box.append(&card_quick_actions);

    // --- 7. Actions & Management Card ---
    let card_actions = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["settings-card"])
        .build();

    let sec_actions = Label::builder()
        .label("ACTIONS & MANAGEMENT")
        .css_classes(["settings-group-header"])
        .xalign(0.0)
        .build();
    card_actions.append(&sec_actions);

    let actions_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(4)
        .build();

    let corner_dd_clone = corner_dropdown.clone();
    let cur_skin_for_save = Rc::clone(&current_skin);
    let cur_scale_for_save = Rc::clone(&current_scale);
    let cur_speed_for_save = Rc::clone(&current_speed);
    let cur_bg_for_save = Rc::clone(&current_bg);
    let cur_txt_for_save = Rc::clone(&current_text);
    let pet_mode_clone = pet_mode_check.clone();
    let mode_dd_clone = mode_dropdown.clone();
    let mon_options_clone = mon_options.clone();
    let mon_dd_clone = mon_dropdown.clone();
    let multi_mon_clone = multi_mon_check.clone();
    let audio_clone = audio_check.clone();
    let sound_dd_clone = sound_dropdown.clone();
    let volume_scale_clone = volume_scale.clone();
    let ignore_clone = ignore_entry.clone();
    let banter_check_clone = banter_check.clone();
    let banter_sound_clone = banter_sound_check.clone();
    let personality_dd_clone = personality_dropdown.clone();
    let banter_min_scale_clone = min_scale.clone();
    let banter_max_scale_clone = max_scale.clone();
    let smart_check_clone = smart_check.clone();
    let cache_alert_clone = cache_alert_check.clone();
    let orphan_alert_clone = orphan_alert_check.clone();
    let failed_services_clone = failed_services_check.clone();
    let custom_actions_for_save = Rc::clone(&custom_actions_rc);

    let save_config = move || {
        let mut cfg = Config::load();
        cfg.custom_actions = custom_actions_for_save.borrow().clone();
        cfg.smart.enabled = smart_check_clone.is_active();
        cfg.smart.pacman_cache_alert = cache_alert_clone.is_active();
        cfg.smart.orphan_alert = orphan_alert_clone.is_active();
        cfg.smart.failed_services_alert = failed_services_clone.is_active();
        cfg.general.corner = match corner_dd_clone.selected() {
            1 => Corner::BottomLeft,
            2 => Corner::TopRight,
            3 => Corner::TopLeft,
            _ => Corner::BottomRight,
        };
        cfg.visual.cat_skin = cur_skin_for_save.get();
        cfg.visual.cat_scale = cur_scale_for_save.get().clamp(1.0, 3.0);
        cfg.visual.animation_speed = cur_speed_for_save.get();
        cfg.visual.bubble_bg_color = cur_bg_for_save.borrow().clone();
        cfg.visual.bubble_text_color = cur_txt_for_save.borrow().clone();
        cfg.visual.always_visible_pet_mode = pet_mode_clone.is_active();
        cfg.general.display_mode = match mode_dd_clone.selected() {
            1 => DisplayMode::DesktopOnly,
            _ => DisplayMode::Overlay,
        };
        let selected_mon = mon_dd_clone.selected() as usize;
        cfg.general.monitor = mon_options_clone
            .get(selected_mon)
            .map(|o| o.id.clone())
            .unwrap_or_else(|| "auto".to_string());
        cfg.general.show_on_all_monitors = multi_mon_clone.is_active();
        cfg.audio.enabled = audio_clone.is_active();
        cfg.audio.sound_effect = match sound_dd_clone.selected() {
            1 => SoundEffect::Chiptune,
            2 => SoundEffect::Mute,
            _ => SoundEffect::Realistic,
        };
        cfg.audio.sound_volume = volume_scale_clone.value().round().clamp(0.0, 100.0) as u32;
        cfg.banter.enabled = banter_check_clone.is_active();
        cfg.banter.play_sound = banter_sound_clone.is_active();
        cfg.banter.personality = match personality_dd_clone.selected() {
            1 => Personality::Cute,
            2 => Personality::Sassy,
            3 => Personality::Lazy,
            _ => Personality::Normal,
        };
        let b_min = banter_min_scale_clone.value().round().clamp(10.0, 600.0) as u32;
        let b_max = banter_max_scale_clone.value().round().clamp(15.0, 1200.0) as u32;
        cfg.banter.interval_min = b_min.min(b_max);
        cfg.banter.interval_max = b_max.max(b_min);
        cfg.updates.ignore_packages = ignore_clone
            .text()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let _ = cfg.save();
    };

    let app_weak = app.downgrade();
    let btn_test = Button::builder()
        .label("▶ Test Notification Now")
        .css_classes(["pixel-button", "pixel-button-primary"])
        .hexpand(true)
        .build();

    let save_fn_test = save_config.clone();
    btn_test.connect_clicked(move |_| {
        save_fn_test();
        let fresh_config = Config::load();
        if let Some(app) = app_weak.upgrade() {
            show_popups(&app, &fresh_config, 7, 2, 0);
        }
    });
    actions_box.append(&btn_test);

    let btn_stop_pet = Button::builder()
        .label("⏹ Stop Running Desktop Pet")
        .css_classes(["pixel-button"])
        .hexpand(true)
        .build();

    btn_stop_pet.connect_clicked(move |_| {
        let _ = crate::ipc::stop_desktop_pet();
    });
    actions_box.append(&btn_stop_pet);

    let btn_uninstall = Button::builder()
        .label("⚠ Purge Data & Deactivate Background Timer")
        .css_classes(["pixel-button", "pixel-button-danger"])
        .hexpand(true)
        .build();

    let win_weak = window.downgrade();
    btn_uninstall.connect_clicked(move |_| {
        let _ = Command::new("systemctl")
            .args(["--user", "disable", "--now", "kittynotify.timer", "kittynotify.service"])
            .status();

        let _ = Config::purge_user_data();

        if let Some(win) = win_weak.upgrade() {
            win.close();
        }
    });
    actions_box.append(&btn_uninstall);
    card_actions.append(&actions_box);

    main_box.append(&card_actions);

    let save_fn_close = save_config.clone();
    let display_opt = gtk4::gdk::Display::default();
    let provider_clone = settings_css_provider.clone();
    window.connect_close_request(move |_| {
        save_fn_close();
        if let Some(display) = &display_opt {
            gtk4::style_context_remove_provider_for_display(display, &provider_clone);
        }
        glib::Propagation::Proceed
    });

    // Vertical-only ScrolledWindow with automatic scrollbar and no horizontal scrolling
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .propagate_natural_width(true)
        .propagate_natural_height(false)
        .vexpand(true)
        .hexpand(true)
        .css_classes(["settings-scrolled"])
        .child(&main_box)
        .build();

    window.set_child(Some(&scrolled));
    window.present();
}
