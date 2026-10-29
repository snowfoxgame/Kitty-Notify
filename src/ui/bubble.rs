use crate::checker::aur::is_command_in_path;
use crate::config::{CatSkin, Corner};
use gtk4::prelude::*;
use gtk4::{
    Box, Button, DrawingArea, EventControllerMotion, Label, Orientation, Overlay, Picture, Widget,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

type ActionCallbackHolder = Rc<std::cell::RefCell<Option<Rc<dyn Fn()>>>>;

#[derive(Clone)]
pub struct SpeechBubbleHandle {
    pub widget: Widget,
    drawing_area: DrawingArea,
    title_label: Label,
    body_label: Label,
    button_row: Box,
    btn_action: Button,
    btn_dismiss: Button,
    action_label_base: Rc<std::cell::RefCell<String>>,
    custom_action_cb: ActionCallbackHolder,
    flatpak_count: Rc<Cell<usize>>,
    repo_count: Rc<Cell<usize>>,
    aur_count: Rc<Cell<usize>>,
    bg_rgb: Rc<std::cell::RefCell<(f64, f64, f64)>>,
    text_color: Rc<std::cell::RefCell<String>>,
    current_title: Rc<std::cell::RefCell<String>>,
    current_message: Rc<std::cell::RefCell<String>>,
}

impl SpeechBubbleHandle {
    pub fn update_counts(&self, repo: usize, aur: usize, flatpak: usize) {
        self.repo_count.set(repo);
        self.aur_count.set(aur);
        self.flatpak_count.set(flatpak);
        *self.custom_action_cb.borrow_mut() = None;
        *self.action_label_base.borrow_mut() = "UPDATE".to_string();
        self.btn_action.set_label("UPDATE");

        let mut parts = Vec::new();
        if repo > 0 {
            parts.push(format!("{} repo", repo));
        }
        if aur > 0 {
            parts.push(format!("{} AUR", aur));
        }
        if flatpak > 0 {
            parts.push(format!("{} Flatpak", flatpak));
        }

        let mem = crate::brain::CatMemory::load();
        let message = if parts.is_empty() {
            "System is fully up to date!".to_string()
        } else if let Some(streak_quip) = mem.get_streak_dialogue() {
            format!("Arch has {} updates ready!\n{}", parts.join(" & "), streak_quip)
        } else {
            format!("Arch has {} updates ready!", parts.join(" & "))
        };
        *self.current_title.borrow_mut() = "Nya! Updates Ready".to_string();
        *self.current_message.borrow_mut() = message;
        self.btn_action.set_visible(true);
        self.btn_dismiss.set_visible(true);
        self.button_row.set_visible(true);
        self.refresh_text();
        self.drawing_area.queue_draw();
    }

    pub fn show_dialogue(&self, quote: &str) {
        *self.current_title.borrow_mut() = "Nya~".to_string();
        *self.current_message.borrow_mut() = quote.to_string();
        *self.custom_action_cb.borrow_mut() = None;
        self.button_row.set_visible(false);
        self.refresh_text();
        self.widget.set_visible(true);
        self.drawing_area.queue_draw();
    }

    pub fn show_helper_prompt(
        &self,
        title: &str,
        message: &str,
        action_label: &str,
        on_action: impl Fn() + 'static,
    ) {
        *self.current_title.borrow_mut() = title.to_string();
        *self.current_message.borrow_mut() = message.to_string();
        *self.action_label_base.borrow_mut() = action_label.to_string();
        self.btn_action.set_label(action_label);
        *self.custom_action_cb.borrow_mut() = Some(Rc::new(on_action));
        self.btn_action.set_visible(true);
        self.btn_dismiss.set_visible(true);
        self.button_row.set_visible(true);
        self.refresh_text();
        self.widget.set_visible(true);
        self.drawing_area.queue_draw();
    }

    pub fn set_custom_message(&self, message: &str) {
        *self.current_message.borrow_mut() = message.to_string();
        self.refresh_text();
    }

    pub fn set_colors(&self, bg_color: &str, text_color: &str) {
        *self.bg_rgb.borrow_mut() = crate::config::parse_hex_color(bg_color);
        *self.text_color.borrow_mut() = text_color.to_string();
        self.refresh_text();
        self.drawing_area.queue_draw();
    }

    fn refresh_text(&self) {
        let text_color = self.text_color.borrow();
        let title = self.current_title.borrow();
        self.title_label.set_markup(&format!(
            "<span foreground=\"{}\">{}</span>",
            text_color,
            glib::markup_escape_text(&title)
        ));
        let message = self.current_message.borrow();
        self.body_label.set_markup(&format!(
            "<span foreground=\"{}\">{}</span>",
            text_color,
            glib::markup_escape_text(&message)
        ));
    }
}

pub fn build_speech_bubble(
    repo_count: usize,
    aur_count: usize,
    flatpak_count: usize,
    corner: Corner,
    bg_color: &str,
    text_color: &str,
    on_dismiss: impl Fn() + 'static + Clone,
) -> SpeechBubbleHandle {
    let overlay = Overlay::new();

    let bg_rgb = Rc::new(std::cell::RefCell::new(crate::config::parse_hex_color(bg_color)));
    let text_color_rc = Rc::new(std::cell::RefCell::new(text_color.to_string()));
    let current_title_rc = Rc::new(std::cell::RefCell::new("Nya! Updates Ready".to_string()));
    let current_message_rc = Rc::new(std::cell::RefCell::new(String::new()));

    let drawing_area = DrawingArea::new();
    drawing_area.set_can_target(false);
    let bg_rgb_draw = Rc::clone(&bg_rgb);
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let rgb = *bg_rgb_draw.borrow();
        draw_pixel_speech_bubble(cr, width as f64, height as f64, corner, rgb);
    });
    overlay.set_child(Some(&drawing_area));

    let content_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["pixel-bubble-content"])
        .build();

    let (pad_top, pad_bottom, pad_left, pad_right) = match corner {
        Corner::BottomRight => (10, 24, 14, 18),
        Corner::BottomLeft => (10, 24, 18, 14),
        Corner::TopRight => (24, 10, 14, 18),
        Corner::TopLeft => (24, 10, 18, 14),
    };
    content_box.set_margin_top(pad_top);
    content_box.set_margin_bottom(pad_bottom);
    content_box.set_margin_start(pad_left);
    content_box.set_margin_end(pad_right);
    content_box.set_width_request(240);

    let title = Label::builder()
        .use_markup(true)
        .css_classes(["pixel-bubble-title"])
        .wrap(true)
        .wrap_mode(pango::WrapMode::Word)
        .max_width_chars(32)
        .xalign(0.0)
        .build();
    content_box.append(&title);

    let mut parts = Vec::new();
    if repo_count > 0 {
        parts.push(format!("{} repo", repo_count));
    }
    if aur_count > 0 {
        parts.push(format!("{} AUR", aur_count));
    }
    if flatpak_count > 0 {
        parts.push(format!("{} Flatpak", flatpak_count));
    }

    let mem = crate::brain::CatMemory::load();
    let initial_message = if parts.is_empty() {
        "System is fully up to date!".to_string()
    } else if let Some(streak_quip) = mem.get_streak_dialogue() {
        format!(
            "Arch has {} updates ready!\n{}",
            parts.join(" & "),
            streak_quip
        )
    } else {
        format!("Arch has {} updates ready!", parts.join(" & "))
    };
    *current_message_rc.borrow_mut() = initial_message;

    let body = Label::builder()
        .use_markup(true)
        .css_classes(["pixel-bubble-body"])
        .wrap(true)
        .wrap_mode(pango::WrapMode::Word)
        .max_width_chars(32)
        .xalign(0.0)
        .build();
    content_box.append(&body);

    let button_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let repo_counter = Rc::new(Cell::new(repo_count));
    let aur_counter = Rc::new(Cell::new(aur_count));
    let flatpak_counter = Rc::new(Cell::new(flatpak_count));

    let action_label_base = Rc::new(std::cell::RefCell::new("UPDATE".to_string()));
    let custom_action_cb: ActionCallbackHolder = Rc::new(std::cell::RefCell::new(None));

    let btn_action = Button::builder()
        .label("UPDATE")
        .width_request(108)
        .css_classes(["pixel-action-btn", "pixel-action-btn-update"])
        .build();

    let motion_update = EventControllerMotion::new();
    let btn_u_enter = btn_action.clone();
    let alb_enter = Rc::clone(&action_label_base);
    motion_update.connect_enter(move |_, _x, _y| {
        let text = alb_enter.borrow();
        btn_u_enter.set_label(&format!("> {} <", *text));
    });
    let btn_u_leave = btn_action.clone();
    let alb_leave = Rc::clone(&action_label_base);
    motion_update.connect_leave(move |_| {
        let text = alb_leave.borrow();
        btn_u_leave.set_label(&text);
    });
    btn_action.add_controller(motion_update);

    let flatpak_for_update = Rc::clone(&flatpak_counter);
    let custom_act_click = Rc::clone(&custom_action_cb);
    let dismiss_clone = on_dismiss.clone();
    btn_action.connect_clicked(move |_| {
        if let Some(cb) = custom_act_click.borrow().as_ref() {
            cb();
            dismiss_clone();
            return;
        }
        let mut mem = crate::brain::CatMemory::load();
        mem.record_update_applied();
        launch_terminal_updater(flatpak_for_update.get());
        dismiss_clone();
    });
    button_row.append(&btn_action);

    let btn_dismiss = Button::builder()
        .label("DISMISS")
        .width_request(108)
        .css_classes(["pixel-action-btn", "pixel-action-btn-dismiss"])
        .build();

    let motion_dismiss = EventControllerMotion::new();
    let btn_d_enter = btn_dismiss.clone();
    motion_dismiss.connect_enter(move |_, _x, _y| {
        btn_d_enter.set_label("> DISMISS <");
    });
    let btn_d_leave = btn_dismiss.clone();
    motion_dismiss.connect_leave(move |_| {
        btn_d_leave.set_label("DISMISS");
    });
    btn_dismiss.add_controller(motion_dismiss);

    let custom_act_dismiss = Rc::clone(&custom_action_cb);
    let repo_c = Rc::clone(&repo_counter);
    let aur_c = Rc::clone(&aur_counter);
    let flatpak_c = Rc::clone(&flatpak_counter);
    let dismiss_cb = on_dismiss.clone();
    btn_dismiss.connect_clicked(move |_| {
        if custom_act_dismiss.borrow().is_none()
            && (repo_c.get() > 0 || aur_c.get() > 0 || flatpak_c.get() > 0)
        {
            let mut mem = crate::brain::CatMemory::load();
            mem.record_dismiss();
        }
        dismiss_cb();
    });
    button_row.append(&btn_dismiss);

    content_box.append(&button_row);

    overlay.add_overlay(&content_box);
    overlay.set_measure_overlay(&content_box, true);

    let handle = SpeechBubbleHandle {
        widget: overlay.upcast(),
        drawing_area,
        title_label: title,
        body_label: body,
        button_row,
        btn_action,
        btn_dismiss,
        action_label_base,
        custom_action_cb,
        flatpak_count: flatpak_counter,
        repo_count: repo_counter,
        aur_count: aur_counter,
        bg_rgb,
        text_color: text_color_rc,
        current_title: current_title_rc,
        current_message: current_message_rc,
    };
    handle.refresh_text();
    handle
}

fn draw_px_rect(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64) {
    cr.rectangle(x.round(), y.round(), w.round(), h.round());
    let _ = cr.fill();
}

pub fn draw_pixel_speech_bubble(
    cr: &cairo::Context,
    w: f64,
    h: f64,
    corner: Corner,
    bg_rgb: (f64, f64, f64),
) {
    let _ = cr.save();

    match corner {
        Corner::BottomRight => {}
        Corner::BottomLeft => {
            cr.translate(w, 0.0);
            cr.scale(-1.0, 1.0);
        }
        Corner::TopRight => {
            cr.translate(0.0, h);
            cr.scale(1.0, -1.0);
        }
        Corner::TopLeft => {
            cr.translate(w, h);
            cr.scale(-1.0, -1.0);
        }
    }

    let p = 2.0;
    let bx = 4.0;
    let by = 4.0;
    let bw = (w - 16.0).max(80.0);
    let bh = (h - 18.0).max(40.0);

    // Tail position: neatly located near the right edge of the bubble
    let tail_r = bx + bw - 6.0 * p;
    let tail_w = 6.0 * p;
    let tail_l = tail_r - tail_w;

    // --- 1. DROP SHADOW (Solid Black) ---
    cr.set_source_rgb(0.0, 0.0, 0.0);

    // Right drop shadow
    draw_px_rect(cr, bx + bw, by + 4.0 * p, 2.0 * p, bh - 6.0 * p);
    // Bottom-right shadow under corner
    draw_px_rect(cr, bx + bw - 2.0 * p, by + bh, 4.0 * p, 2.0 * p);

    // Bottom shadow left of the tail
    draw_px_rect(cr, bx + 4.0 * p, by + bh, (tail_l - (bx + 4.0 * p)).max(0.0), 2.0 * p);
    // Bottom shadow right of the tail
    draw_px_rect(cr, tail_r + p, by + bh, ((bx + bw - 2.0 * p) - (tail_r + p)).max(0.0), 2.0 * p);

    // Tail shadow
    draw_px_rect(cr, tail_r + p, by + bh + p, 2.0 * p, 3.0 * p);
    draw_px_rect(cr, tail_r + 2.0 * p, by + bh + 3.0 * p, 2.0 * p, 4.0 * p);
    draw_px_rect(cr, tail_l + 3.0 * p, by + bh + 7.0 * p, 5.0 * p, 2.0 * p);

    // --- 2. INTERIOR FILL (Custom Color) ---
    cr.set_source_rgb(bg_rgb.0, bg_rgb.1, bg_rgb.2);
    // Main bubble body
    draw_px_rect(cr, bx + p, by + p, bw - 2.0 * p, bh - 2.0 * p);

    // Tail fill (strictly inside borders)
    draw_px_rect(cr, tail_l, by + bh - p, tail_w, 2.0 * p);
    draw_px_rect(cr, tail_l + p, by + bh + p, tail_w - p, 2.0 * p);
    draw_px_rect(cr, tail_l + 2.0 * p, by + bh + 3.0 * p, tail_w - 2.0 * p, 3.0 * p);

    // --- 3. BLACK BORDERS & STEPPED CORNERS ---
    cr.set_source_rgb(0.0, 0.0, 0.0);

    // Top border
    draw_px_rect(cr, bx + 3.0 * p, by, bw - 6.0 * p, p);
    // Left border
    draw_px_rect(cr, bx, by + 3.0 * p, p, bh - 6.0 * p);
    // Right border
    draw_px_rect(cr, bx + bw - p, by + 3.0 * p, p, bh - 6.0 * p);

    // Bottom border segments
    draw_px_rect(cr, bx + 3.0 * p, by + bh - p, (tail_l - (bx + 3.0 * p)).max(0.0), p);
    draw_px_rect(cr, tail_r, by + bh - p, ((bx + bw - 3.0 * p) - tail_r).max(0.0), p);

    // Stepped corners (TL, TR, BL, BR)
    // TL
    draw_px_rect(cr, bx + p, by + p, p, p);
    draw_px_rect(cr, bx + 2.0 * p, by, p, p);
    draw_px_rect(cr, bx, by + 2.0 * p, p, p);

    // TR
    draw_px_rect(cr, bx + bw - 2.0 * p, by + p, p, p);
    draw_px_rect(cr, bx + bw - 3.0 * p, by, p, p);
    draw_px_rect(cr, bx + bw - p, by + 2.0 * p, p, p);

    // BL
    draw_px_rect(cr, bx + p, by + bh - 2.0 * p, p, p);
    draw_px_rect(cr, bx + 2.0 * p, by + bh - p, p, p);
    draw_px_rect(cr, bx, by + bh - 3.0 * p, p, p);

    // BR
    draw_px_rect(cr, bx + bw - 2.0 * p, by + bh - 2.0 * p, p, p);
    draw_px_rect(cr, bx + bw - 3.0 * p, by + bh - p, p, p);
    draw_px_rect(cr, bx + bw - p, by + bh - 3.0 * p, p, p);

    // --- 4. TAIL BLACK OUTLINE ---
    // Left wall of tail (stepping right/down)
    draw_px_rect(cr, tail_l - p, by + bh - p, p, 2.0 * p);
    draw_px_rect(cr, tail_l, by + bh + p, p, 2.0 * p);
    draw_px_rect(cr, tail_l + p, by + bh + 3.0 * p, p, 3.0 * p);

    // Right wall of tail (stepping right/down)
    draw_px_rect(cr, tail_r, by + bh, p, 2.0 * p);
    draw_px_rect(cr, tail_r + p, by + bh + 2.0 * p, p, 4.0 * p);

    // Tail tip (bottom horizontal border)
    draw_px_rect(cr, tail_l + 2.0 * p, by + bh + 6.0 * p, (tail_r + p) - (tail_l + 2.0 * p), p);

    let _ = cr.restore();
}

pub const BASE_FRAME_DELAY_MS: u64 = 220;

fn schedule_next_cat_frame(
    pic_weak: glib::WeakRef<Picture>,
    textures: Rc<Vec<gtk4::gdk::Texture>>,
    frame_idx: Rc<Cell<usize>>,
    speed_cell: Rc<Cell<f32>>,
    is_initial: bool,
) {
    let speed = speed_cell.get().clamp(0.2, 5.0);
    let cur = frame_idx.get();
    let num_frames = textures.len();
    if num_frames <= 1 {
        return;
    }
    let next = (cur + 1) % num_frames;

    // Delay before switching to next frame:
    // When cur == 0 and not initial, we have just completed a full idle cycle and are resting at frame 0.
    // Hold the neutral resting frame for a random pause of 2.0 to 4.0 seconds (scaled inversely by speed).
    let delay_ms = if cur == 0 && !is_initial {
        let pause = glib::random_int_range(2000, 4001) as f32;
        ((pause / speed).round() as u64).max(50)
    } else {
        (((BASE_FRAME_DELAY_MS as f32) / speed).round() as u64).max(20)
    };

    glib::timeout_add_local_once(Duration::from_millis(delay_ms), move || {
        if let Some(pic) = pic_weak.upgrade() {
            frame_idx.set(next);
            pic.set_paintable(Some(&textures[next]));
            pic.queue_draw();
            schedule_next_cat_frame(pic_weak, textures, frame_idx, speed_cell, false);
        }
    });
}

fn schedule_next_standard_cat_frame(
    pic_weak: glib::WeakRef<Picture>,
    textures: Rc<crate::assets::StandardCatTextures>,
    ctrl_rc: Rc<RefCell<crate::ui::animator::StandardCatController>>,
    speed_cell: Rc<Cell<f32>>,
    is_initial: bool,
) {
    let speed = speed_cell.get().clamp(0.2, 5.0);
    let (state, frame, delay_ms) = {
        let mut ctrl = ctrl_rc.borrow_mut();
        ctrl.advance_frame(&textures, speed, is_initial)
    };

    let delay = Duration::from_millis(delay_ms);

    glib::timeout_add_local_once(delay, move || {
        if let Some(pic) = pic_weak.upgrade() {
            let tex_slice = textures.get(state);
            if let Some(tex) = tex_slice.get(frame) {
                pic.set_paintable(Some(tex));
                pic.queue_draw();
            }
            schedule_next_standard_cat_frame(pic_weak, textures, ctrl_rc, speed_cell, false);
        }
    });
}

#[allow(dead_code)]
pub fn build_animated_cat_sprite(
    skin: CatSkin,
    scale: f32,
    on_pet: Option<std::boxed::Box<dyn Fn() + 'static>>,
) -> Box {
    build_animated_cat_sprite_with_speed(skin, scale, None, on_pet)
}

pub fn build_animated_cat_sprite_with_speed(
    skin: CatSkin,
    scale: f32,
    speed_cell: Option<Rc<Cell<f32>>>,
    on_pet: Option<std::boxed::Box<dyn Fn() + 'static>>,
) -> Box {
    build_animated_cat_sprite_with_controller(skin, scale, speed_cell, on_pet, None)
}

pub fn build_animated_cat_sprite_with_controller(
    skin: CatSkin,
    scale: f32,
    speed_cell: Option<Rc<Cell<f32>>>,
    on_pet: Option<std::boxed::Box<dyn Fn() + 'static>>,
    ctrl_opt: Option<Rc<RefCell<crate::ui::animator::StandardCatController>>>,
) -> Box {
    let size = (64.0 * scale.clamp(1.0, 3.0)).round() as i32;
    let picture = Picture::builder()
        .can_shrink(false)
        .width_request(size)
        .height_request(size)
        .css_classes(["cat-sprite"])
        .valign(gtk4::Align::End)
        .build();

    let speed = speed_cell.unwrap_or_else(|| Rc::new(Cell::new(1.0)));

    match skin {
        CatSkin::IdleCat => {
            let textures = Rc::new(crate::assets::load_all_standard_cat_textures(scale));
            if let Some(first) = textures.idle.first() {
                picture.set_paintable(Some(first));
            }
            let ctrl = ctrl_opt.unwrap_or_else(|| {
                Rc::new(RefCell::new(crate::ui::animator::StandardCatController::new()))
            });
            let pic_weak = picture.downgrade();
            schedule_next_standard_cat_frame(pic_weak, textures, ctrl, speed, true);
        }
        CatSkin::CartonCat | CatSkin::DraculaCat => {
            let textures = crate::assets::load_skin_textures(skin, scale);
            if let Some(first) = textures.first() {
                picture.set_paintable(Some(first));
            }
            if textures.len() > 1 {
                let textures_rc = Rc::new(textures);
                let frame_idx = Rc::new(Cell::new(0));
                let pic_weak = picture.downgrade();
                schedule_next_cat_frame(pic_weak, textures_rc, frame_idx, speed, true);
            }
        }
    }

    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(8)
        .valign(gtk4::Align::End)
        .halign(gtk4::Align::Center)
        .css_classes(["cat-sprite-container"])
        .build();
    container.append(&picture);

    if let Some(pet_cb) = on_pet {
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_, _, _, _| {
            pet_cb();
        });
        container.add_controller(gesture);
    }

    container
}

pub fn launch_in_terminal(command_str: &str) {
    crate::actions::launch_in_terminal(command_str);
}

pub fn launch_terminal_updater(flatpak_count: usize) {
    let mut update_cmd = if is_command_in_path("paru") {
        "paru".to_string()
    } else if is_command_in_path("yay") {
        "yay".to_string()
    } else {
        "sudo pacman -Syu".to_string()
    };

    if flatpak_count > 0 {
        update_cmd.push_str(" && flatpak update");
    }

    launch_in_terminal(&update_cmd);
}
