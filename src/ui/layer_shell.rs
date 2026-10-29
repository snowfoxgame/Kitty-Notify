use crate::audio::play_meow;
use crate::config::{Config, Corner};
#[cfg(feature = "wayland")]
use crate::config::DisplayMode;
use crate::ui::animator::{AnimPhase, AnimationState};
use crate::ui::bubble::{
    build_animated_cat_sprite_with_controller, build_speech_bubble, SpeechBubbleHandle,
};
use gtk4::gdk::{Display, Monitor};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Fixed};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

#[cfg(feature = "wayland")]
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

type ActionCallback = std::boxed::Box<dyn Fn() + 'static>;
type TapHandlerHolder = Rc<std::cell::RefCell<Option<ActionCallback>>>;

#[derive(Clone)]
pub struct PetController {
    bubble_handle: SpeechBubbleHandle,
    bubble_visible: Rc<Cell<bool>>,
    apply_input_mask: Rc<dyn Fn()>,
    audio_cfg: crate::config::AudioConfig,
    banter_cfg: crate::config::BanterConfig,
    smart_cfg: crate::config::SmartConfig,
    auto_dismiss_seconds: u32,
    is_update_active: Rc<Cell<bool>>,
    dialogue_timer: Rc<Cell<Option<glib::SourceId>>>,
    is_alive: Rc<Cell<bool>>,
    window_weak: glib::WeakRef<ApplicationWindow>,
    pub cat_controller: Option<Rc<std::cell::RefCell<crate::ui::animator::StandardCatController>>>,
}

impl PetController {
    pub fn is_sleeping(&self) -> bool {
        if let Some(sc) = &self.cat_controller {
            sc.borrow().current_state == crate::ui::animator::StandardCatState::Sleeping
        } else {
            false
        }
    }

    pub fn trigger_update(&self, repo: usize, aur: usize, flatpak: usize) {
        if let Some(source_id) = self.dialogue_timer.take() {
            source_id.remove();
        }
        play_meow(&self.audio_cfg);
        let has_any = repo > 0 || aur > 0 || flatpak > 0;
        self.is_update_active.set(has_any);
        self.bubble_handle.update_counts(repo, aur, flatpak);
        self.bubble_handle.widget.set_visible(true);
        self.bubble_visible.set(true);
        (self.apply_input_mask)();

        if self.auto_dismiss_seconds > 0 {
            let ctrl = self.clone();
            glib::timeout_add_local_once(
                Duration::from_secs(self.auto_dismiss_seconds as u64),
                move || {
                    if ctrl.is_update_active.get() {
                        let mut mem = crate::brain::CatMemory::load();
                        mem.record_dismiss();
                    }
                    ctrl.dismiss();
                },
            );
        }
    }

    pub fn dismiss(&self) {
        if let Some(source_id) = self.dialogue_timer.take() {
            source_id.remove();
        }
        self.is_update_active.set(false);
        self.bubble_handle.widget.set_visible(false);
        self.bubble_visible.set(false);
        (self.apply_input_mask)();
    }

    pub fn trigger_tap_or_banter(&self) {
        if self.is_update_active.get() {
            return;
        }
        let quote = crate::ui::dialogue::get_random_quip();
        self.bubble_handle.show_dialogue(&quote);
        self.bubble_visible.set(true);
        (self.apply_input_mask)();

        if self.banter_cfg.play_sound && self.audio_cfg.enabled {
            crate::audio::play_tap_meow(&self.audio_cfg);
        }

        if let Some(source_id) = self.dialogue_timer.take() {
            source_id.remove();
        }

        let ctrl_hide = self.clone();
        let timer_id = glib::timeout_add_local_once(Duration::from_millis(3500), move || {
            ctrl_hide.dialogue_timer.set(None);
            if !ctrl_hide.is_update_active.get() {
                ctrl_hide.bubble_handle.widget.set_visible(false);
                ctrl_hide.bubble_visible.set(false);
                (ctrl_hide.apply_input_mask)();
            }
        });
        self.dialogue_timer.set(Some(timer_id));
    }

    pub fn start_banter_loop(&self) {
        if !self.banter_cfg.enabled {
            return;
        }
        let ctrl = self.clone();
        schedule_next_banter(ctrl);
    }
}

fn schedule_next_banter(ctrl: PetController) {
    if !ctrl.banter_cfg.enabled || !ctrl.is_alive.get() || ctrl.window_weak.upgrade().is_none() {
        return;
    }

    let min_s = ctrl
        .banter_cfg
        .interval_min
        .min(ctrl.banter_cfg.interval_max)
        .max(5);
    let max_s = ctrl.banter_cfg.interval_max.max(min_s).max(min_s + 1);
    let delay_s = glib::random_int_range(min_s as i32, (max_s + 1) as i32) as u64;

    glib::timeout_add_local_once(Duration::from_secs(delay_s), move || {
        if !ctrl.is_alive.get() || ctrl.window_weak.upgrade().is_none() {
            return;
        }

        // Guard: If pet is currently sleeping (nighttime loaf), suppress banter speech bubbles entirely!
        if ctrl.is_sleeping() {
            schedule_next_banter(ctrl);
            return;
        }

        // Guard: If an update prompt is active on screen, don't interrupt it; retry later
        if ctrl.is_update_active.get() {
            schedule_next_banter(ctrl);
            return;
        }

        let mut memory = crate::brain::CatMemory::load();
        memory.check_and_record_late_night();

        // 1. Crash recovery dialogue (unclean shutdown marker evaluation)
        if let Some(recovery) = memory.get_recovery_dialogue() {
            ctrl.bubble_handle.show_dialogue(&recovery);
            ctrl.bubble_visible.set(true);
            (ctrl.apply_input_mask)();

            if ctrl.banter_cfg.play_sound && ctrl.audio_cfg.enabled {
                crate::audio::play_tap_meow(&ctrl.audio_cfg);
            }

            let ctrl_hide = ctrl.clone();
            let timer_id = glib::timeout_add_local_once(Duration::from_millis(4500), move || {
                ctrl_hide.dialogue_timer.set(None);
                if !ctrl_hide.is_alive.get() || ctrl_hide.window_weak.upgrade().is_none() {
                    return;
                }
                if !ctrl_hide.is_update_active.get() {
                    ctrl_hide.bubble_handle.widget.set_visible(false);
                    ctrl_hide.bubble_visible.set(false);
                    (ctrl_hide.apply_input_mask)();
                }
                schedule_next_banter(ctrl_hide);
            });
            ctrl.dialogue_timer.set(Some(timer_id));
            return;
        }

        // 2. Proactive system helper evaluation (Pacman cache, orphans, systemd units)
        let cfg_dummy = crate::config::Config {
            smart: ctrl.smart_cfg.clone(),
            ..Default::default()
        };
        if let Some(alert) = crate::brain::check_system_helpers(&cfg_dummy, &mut memory) {
            let cmd_to_run = alert.command.clone();
            let ctrl_for_action = ctrl.clone();
            ctrl.bubble_handle.show_helper_prompt(
                &alert.title,
                &alert.message,
                &alert.action_label,
                move || {
                    crate::ui::bubble::launch_in_terminal(&cmd_to_run);
                    ctrl_for_action.dismiss();
                    schedule_next_banter(ctrl_for_action.clone());
                },
            );
            ctrl.bubble_visible.set(true);
            (ctrl.apply_input_mask)();

            if ctrl.banter_cfg.play_sound && ctrl.audio_cfg.enabled {
                crate::audio::play_tap_meow(&ctrl.audio_cfg);
            }
            return;
        }

        // Sample live hardware metrics once per banter tick
        let snapshot = crate::ui::dialogue::SystemSnapshot::gather();

        // 3. Behavioral dialogue engine, habit-aware dialogue, or context-aware system dialogue
        #[cfg(feature = "smart")]
        let quote = crate::brain::evaluate_behavioral_dialogue_with_snapshot(&memory, ctrl.banter_cfg.personality, &snapshot)
            .or_else(|| memory.get_habit_dialogue())
            .unwrap_or_else(|| crate::ui::dialogue::get_smart_dialogue_for_personality_with_snapshot(ctrl.banter_cfg.personality, &snapshot));

        #[cfg(not(feature = "smart"))]
        let quote = crate::ui::dialogue::get_smart_dialogue_for_personality_with_snapshot(ctrl.banter_cfg.personality, &snapshot);

        // Track running app habits
        memory.record_app_seen(&snapshot.top_process_name);

        // Display quote in speech bubble
        ctrl.bubble_handle.show_dialogue(&quote);
        ctrl.bubble_visible.set(true);
        (ctrl.apply_input_mask)();

        // Play quiet feedback sound if enabled
        if ctrl.banter_cfg.play_sound && ctrl.audio_cfg.enabled {
            crate::audio::play_tap_meow(&ctrl.audio_cfg);
        }

        // Auto-hide after 4 seconds and schedule next banter
        let ctrl_hide = ctrl.clone();
        let timer_id = glib::timeout_add_local_once(Duration::from_millis(4000), move || {
            ctrl_hide.dialogue_timer.set(None);
            if !ctrl_hide.is_alive.get() || ctrl_hide.window_weak.upgrade().is_none() {
                return;
            }
            if !ctrl_hide.is_update_active.get() {
                ctrl_hide.bubble_handle.widget.set_visible(false);
                ctrl_hide.bubble_visible.set(false);
                (ctrl_hide.apply_input_mask)();
            }
            schedule_next_banter(ctrl_hide);
        });
        ctrl.dialogue_timer.set(Some(timer_id));
    });
}

pub fn spawn_popup_window(
    app: &Application,
    config: &Config,
    _monitor: Option<&Monitor>,
    repo_count: usize,
    aur_count: usize,
    flatpak_count: usize,
) -> ApplicationWindow {
    let cat_size = (64.0 * (config.visual.cat_scale.clamp(1.0, 3.0) as f64)).round();
    let bubble_w = 280.0;
    let bubble_h = 130.0;
    let win_w = (bubble_w + cat_size + 36.0).max(380.0);
    let win_h = (bubble_h + cat_size + 48.0).max(260.0);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("KittyNotify")
        .decorated(false)
        .default_width(win_w as i32)
        .default_height(win_h as i32)
        .css_classes(["pixel-window"])
        .focusable(false)
        .can_focus(false)
        .build();

    #[cfg(feature = "wayland")]
    if gtk4_layer_shell::is_supported() {
        window.init_layer_shell();

        match config.general.display_mode {
            DisplayMode::Overlay => window.set_layer(Layer::Overlay),
            DisplayMode::DesktopOnly => window.set_layer(Layer::Bottom),
        }

        window.set_keyboard_mode(KeyboardMode::None);

        match config.general.corner {
            Corner::BottomRight => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Top, false);
                window.set_anchor(Edge::Left, false);
                window.set_margin(Edge::Bottom, 16);
                window.set_margin(Edge::Right, 16);
                window.set_margin(Edge::Top, 0);
                window.set_margin(Edge::Left, 0);
            }
            Corner::BottomLeft => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
                window.set_anchor(Edge::Top, false);
                window.set_anchor(Edge::Right, false);
                window.set_margin(Edge::Bottom, 16);
                window.set_margin(Edge::Left, 16);
                window.set_margin(Edge::Top, 0);
                window.set_margin(Edge::Right, 0);
            }
            Corner::TopRight => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Bottom, false);
                window.set_anchor(Edge::Left, false);
                window.set_margin(Edge::Top, 16);
                window.set_margin(Edge::Right, 16);
                window.set_margin(Edge::Bottom, 0);
                window.set_margin(Edge::Left, 0);
            }
            Corner::TopLeft => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Left, true);
                window.set_anchor(Edge::Bottom, false);
                window.set_anchor(Edge::Right, false);
                window.set_margin(Edge::Top, 16);
                window.set_margin(Edge::Left, 16);
                window.set_margin(Edge::Bottom, 0);
                window.set_margin(Edge::Right, 0);
            }
        }

        if let Some(mon) = _monitor {
            window.set_monitor(Some(mon));
        }
    }

    let (cat_x, cat_y, bubble_x, bubble_y) = match config.general.corner {
        Corner::BottomRight => {
            let cx = win_w - cat_size - 16.0;
            let cy = win_h - cat_size - 18.0;
            let bx = (cx - bubble_w - 4.0).max(12.0);
            let by = (cy - bubble_h + (cat_size * 0.3)).max(10.0);
            (cx, cy, bx, by)
        }
        Corner::BottomLeft => {
            let cx = 16.0;
            let cy = win_h - cat_size - 18.0;
            let bx = cx + cat_size + 4.0;
            let by = (cy - bubble_h + (cat_size * 0.3)).max(10.0);
            (cx, cy, bx, by)
        }
        Corner::TopRight => {
            let cx = win_w - cat_size - 16.0;
            let cy = 16.0;
            let bx = (cx - bubble_w - 4.0).max(12.0);
            let by = cy + (cat_size * 0.25).max(10.0);
            (cx, cy, bx, by)
        }
        Corner::TopLeft => {
            let cx = 16.0;
            let cy = 16.0;
            let bx = cx + cat_size + 4.0;
            let by = cy + (cat_size * 0.25).max(10.0);
            (cx, cy, bx, by)
        }
    };

    let is_pet = config.visual.always_visible_pet_mode;
    let has_updates = repo_count > 0 || aur_count > 0 || flatpak_count > 0;
    let is_update_active = Rc::new(Cell::new(if is_pet { has_updates } else { true }));
    let dialogue_timer: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
    let bubble_visible = Rc::new(Cell::new(if is_pet { has_updates } else { true }));

    let pet_tap_handler: TapHandlerHolder = Rc::new(std::cell::RefCell::new(None));
    let tap_dispatch = Rc::clone(&pet_tap_handler);

    // Click gesture is strictly restricted to Desktop Pet mode only (always_visible = true)
    let on_pet_cb: Option<ActionCallback> = if is_pet {
        Some(std::boxed::Box::new(move || {
            if let Some(handler) = tap_dispatch.borrow().as_ref() {
                handler();
            }
        }))
    } else {
        None
    };

    let speed_cell = Rc::new(Cell::new(config.visual.animation_speed.multiplier()));

    let std_cat_ctrl = if config.visual.cat_skin == crate::config::CatSkin::IdleCat {
        Some(Rc::new(std::cell::RefCell::new(
            crate::ui::animator::StandardCatController::new(),
        )))
    } else {
        None
    };

    let cat_sprite = build_animated_cat_sprite_with_controller(
        config.visual.cat_skin,
        config.visual.cat_scale,
        Some(Rc::clone(&speed_cell)),
        on_pet_cb,
        std_cat_ctrl.clone(),
    );

    match config.general.corner {
        Corner::BottomRight => {}
        Corner::BottomLeft => cat_sprite.add_css_class("cat-flipped-x"),
        Corner::TopRight => cat_sprite.add_css_class("cat-flipped-y"),
        Corner::TopLeft => cat_sprite.add_css_class("cat-flipped-xy"),
    }

    let cat_rect = cairo::RectangleInt::new(
        cat_x as i32,
        cat_y as i32,
        cat_size as i32,
        (cat_size + 20.0) as i32,
    );

    let win_weak_mask = window.downgrade();
    let b_vis_mask = Rc::clone(&bubble_visible);
    let bubble_holder: Rc<std::cell::RefCell<Option<gtk4::Widget>>> =
        Rc::new(std::cell::RefCell::new(None));
    let bh_mask = Rc::clone(&bubble_holder);
    let apply_input_mask: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(win) = win_weak_mask.upgrade() {
            if let Some(surface) = win.surface() {
                let region = cairo::Region::create();
                let _ = region.union_rectangle(&cat_rect);
                if b_vis_mask.get() {
                    let (cur_w, cur_h) = if let Some(w) = bh_mask.borrow().as_ref() {
                        (w.width().max(bubble_w as i32), w.height().max((bubble_h + 14.0) as i32))
                    } else {
                        (bubble_w as i32, (bubble_h + 14.0) as i32)
                    };
                    let dynamic_bubble_rect = cairo::RectangleInt::new(
                        bubble_x as i32,
                        bubble_y as i32,
                        cur_w,
                        cur_h + 10,
                    );
                    let _ = region.union_rectangle(&dynamic_bubble_rect);
                }
                surface.set_input_region(Some(&region));
            }
        }
    });

    let anim_state = Rc::new(std::cell::RefCell::new(AnimationState::new_with_height(
        config.general.corner,
        config.general.auto_dismiss_seconds,
        win_h,
    )));

    let anim_dismiss = Rc::clone(&anim_state);
    let b_vis_dismiss = Rc::clone(&bubble_visible);
    let mask_for_dismiss = Rc::clone(&apply_input_mask);
    let is_upd_dismiss = Rc::clone(&is_update_active);
    let dt_dismiss = Rc::clone(&dialogue_timer);
    let bh_dismiss = Rc::clone(&bubble_holder);

    let bubble_handle = build_speech_bubble(
        repo_count,
        aur_count,
        flatpak_count,
        config.general.corner,
        &config.visual.bubble_bg_color,
        &config.visual.bubble_text_color,
        move || {
            if is_pet {
                if let Some(source_id) = dt_dismiss.take() {
                    source_id.remove();
                }
                is_upd_dismiss.set(false);
                if let Some(w) = bh_dismiss.borrow().as_ref() {
                    w.set_visible(false);
                }
                b_vis_dismiss.set(false);
                mask_for_dismiss();
            } else {
                let now_us = glib::monotonic_time();
                anim_dismiss.borrow_mut().trigger_slide_out(now_us);
            }
        },
    );

    let bubble_widget = bubble_handle.widget.clone();
    *bubble_holder.borrow_mut() = Some(bubble_widget.clone());
    bubble_widget.set_visible(bubble_visible.get());

    let layout_container = Fixed::new();
    layout_container.set_width_request(win_w as i32);
    layout_container.set_height_request(win_h as i32);
    layout_container.put(&cat_sprite, cat_x, cat_y);
    layout_container.put(&bubble_widget, bubble_x, bubble_y);

    if is_pet {
        let is_upd_tap = Rc::clone(&is_update_active);
        let b_handle_tap = bubble_handle.clone();
        let b_vis_tap = Rc::clone(&bubble_visible);
        let mask_tap = Rc::clone(&apply_input_mask);
        let audio_cfg_tap = config.audio.clone();
        let layout_hop = layout_container.clone();
        let cat_sprite_hop = cat_sprite.clone();
        let dt_tap = Rc::clone(&dialogue_timer);
        let hop_active = Rc::new(Cell::new(false));

        crate::ui::context_menu::attach_pet_context_menu(
            app,
            &cat_sprite,
            config.general.corner,
            Rc::clone(&apply_input_mask),
            window.downgrade(),
        );

        *pet_tap_handler.borrow_mut() = Some(std::boxed::Box::new(move || {
            // Guard: If an update notification is currently active on screen, do NOT override it!
            if is_upd_tap.get() {
                return;
            }

            // 1. Play gentle audio feedback (purr / chirpy meow)
            crate::audio::play_tap_meow(&audio_cfg_tap);

            // 2. Play hop / bounce reaction animation
            if !hop_active.get() {
                hop_active.set(true);
                let hop_layout = layout_hop.clone();
                let hop_sprite = cat_sprite_hop.clone();
                let hop_flag = Rc::clone(&hop_active);
                let start_us = glib::monotonic_time();
                let total_us: f64 = 350_000.0;
                let hop_height: f64 = 16.0;

                glib::timeout_add_local(Duration::from_millis(16), move || {
                    let elapsed = (glib::monotonic_time() - start_us) as f64;
                    if elapsed >= total_us {
                        hop_layout.move_(&hop_sprite, cat_x, cat_y);
                        hop_flag.set(false);
                        glib::ControlFlow::Break
                    } else {
                        let p = (elapsed / total_us).clamp(0.0, 1.0);
                        let dy = -hop_height * 4.0 * p * (1.0 - p);
                        hop_layout.move_(&hop_sprite, cat_x, cat_y + dy);
                        glib::ControlFlow::Continue
                    }
                });
            }

            // 3. Display random quote from dialogue pool
            let quip = crate::ui::dialogue::get_random_quip();
            b_handle_tap.show_dialogue(&quip);
            b_vis_tap.set(true);
            mask_tap();

            // 4. Reset & start 3.5s auto-dismiss timer
            if let Some(source_id) = dt_tap.take() {
                source_id.remove();
            }

            let b_handle_hide = b_handle_tap.clone();
            let b_vis_hide = Rc::clone(&b_vis_tap);
            let mask_hide = Rc::clone(&mask_tap);
            let is_upd_hide = Rc::clone(&is_upd_tap);
            let dt_cell = Rc::clone(&dt_tap);

            let new_timer_id = glib::timeout_add_local_once(Duration::from_millis(3500), move || {
                dt_cell.set(None);
                if !is_upd_hide.get() {
                    b_handle_hide.widget.set_visible(false);
                    b_vis_hide.set(false);
                    mask_hide();
                }
            });
            dt_tap.set(Some(new_timer_id));
        }));
    }

    let root_fixed = Fixed::new();
    root_fixed.set_width_request(win_w as i32);
    root_fixed.set_height_request(win_h as i32);

    if is_pet {
        crate::ipc::record_pet_pid();
        root_fixed.put(&layout_container, 0.0, 0.0);

        let mask_init = Rc::clone(&apply_input_mask);
        let win_weak_tick = window.downgrade();
        let surface_configured = Rc::new(Cell::new(false));
        let sc_clone = Rc::clone(&surface_configured);
        root_fixed.add_tick_callback(move |_, _| {
            if !sc_clone.get() {
                if let Some(win) = win_weak_tick.upgrade() {
                    if win.surface().is_some() {
                        mask_init();
                        sc_clone.set(true);
                        return glib::ControlFlow::Break;
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        let is_alive = Rc::new(Cell::new(true));

        let controller = PetController {
            bubble_handle,
            bubble_visible: Rc::clone(&bubble_visible),
            apply_input_mask: Rc::clone(&apply_input_mask),
            audio_cfg: config.audio.clone(),
            banter_cfg: config.banter.clone(),
            smart_cfg: config.smart.clone(),
            auto_dismiss_seconds: config.general.auto_dismiss_seconds,
            is_update_active: Rc::clone(&is_update_active),
            dialogue_timer: Rc::clone(&dialogue_timer),
            is_alive: Rc::clone(&is_alive),
            window_weak: window.downgrade(),
            cat_controller: std_cat_ctrl,
        };

        if config.banter.enabled {
            controller.start_banter_loop();
        }

        if has_updates && config.general.auto_dismiss_seconds > 0 {
            let ctrl_timeout = controller.clone();
            glib::timeout_add_local_once(
                Duration::from_secs(config.general.auto_dismiss_seconds as u64),
                move || {
                    if ctrl_timeout.is_update_active.get() {
                        let mut mem = crate::brain::CatMemory::load();
                        mem.record_dismiss();
                    }
                    ctrl_timeout.dismiss();
                },
            );
        }

        let (tx, rx) = async_channel::unbounded::<String>();
        crate::ipc::start_ipc_server(tx);

        let ctrl_ipc = controller.clone();
        let speed_ipc = Rc::clone(&speed_cell);
        glib::spawn_future_local(async move {
            while let Ok(cmd) = rx.recv().await {
                tracing::info!("Pet IPC received command: {}", cmd);
                if cmd == "test" {
                    ctrl_ipc.trigger_update(7, 2, 0);
                } else if cmd == "tap" || cmd == "banter" {
                    ctrl_ipc.trigger_tap_or_banter();
                } else if cmd.starts_with("notify ") {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    let repo = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let aur = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let flatpak = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
                    ctrl_ipc.trigger_update(repo, aur, flatpak);
                } else if cmd.starts_with("speed ") {
                    if let Some(val_str) = cmd.strip_prefix("speed ") {
                        let clean = val_str.trim().to_lowercase();
                        let spd_opt = if clean == "slow" || clean == "half" {
                            Some(0.5)
                        } else if clean == "normal" || clean == "default" {
                            Some(1.0)
                        } else if clean == "fast" || clean == "double" {
                            Some(2.0)
                        } else {
                            clean.trim_end_matches('x').parse::<f32>().ok()
                        };
                        if let Some(spd) = spd_opt {
                            let clamped = spd.clamp(0.2, 5.0);
                            speed_ipc.set(clamped);
                            tracing::info!("IPC: Updated pet animation speed to {:.2}x", clamped);
                        }
                    }
                } else if cmd.starts_with("state ") {
                    if let Some(state_name) = cmd.strip_prefix("state ") {
                        let clean = state_name.trim();
                        if clean == "auto" || clean == "reset" {
                            if let Some(sc) = &ctrl_ipc.cat_controller {
                                sc.borrow_mut().clear_forced_state();
                                tracing::info!("IPC: Reset Standard Cat state to auto");
                            }
                        } else if let Some(st) = crate::ui::animator::StandardCatState::from_str(clean) {
                            if let Some(sc) = &ctrl_ipc.cat_controller {
                                sc.borrow_mut().set_forced_state(st);
                                tracing::info!("IPC: Set Standard Cat state to {:?}", st);
                            }
                        }
                    }
                } else if cmd == "excited" {
                    if let Some(sc) = &ctrl_ipc.cat_controller {
                        sc.borrow_mut().trigger_excited(std::time::Instant::now());
                        tracing::info!("IPC: Triggered Standard Cat excited state");
                    }
                } else if let Some(msg) = cmd.strip_prefix("say ") {
                    ctrl_ipc.bubble_handle.show_dialogue(msg.trim());
                    ctrl_ipc.bubble_visible.set(true);
                    (ctrl_ipc.apply_input_mask)();
                    if ctrl_ipc.banter_cfg.play_sound && ctrl_ipc.audio_cfg.enabled {
                        crate::audio::play_tap_meow(&ctrl_ipc.audio_cfg);
                    }
                    if let Some(source_id) = ctrl_ipc.dialogue_timer.take() {
                        source_id.remove();
                    }
                    let ctrl_hide = ctrl_ipc.clone();
                    let timer_id = glib::timeout_add_local_once(Duration::from_millis(4500), move || {
                        ctrl_hide.dialogue_timer.set(None);
                        if !ctrl_hide.is_update_active.get() {
                            ctrl_hide.bubble_handle.widget.set_visible(false);
                            ctrl_hide.bubble_visible.set(false);
                            (ctrl_hide.apply_input_mask)();
                        }
                    });
                    ctrl_ipc.dialogue_timer.set(Some(timer_id));
                } else if let Some(action_str) = cmd.strip_prefix("action ") {
                    let clean = action_str.trim();
                    let cfg = Config::load();
                    if let Some(act) = cfg.custom_actions.iter().find(|a| a.name.eq_ignore_ascii_case(clean)) {
                        crate::actions::execute_action(&act.command);
                    } else {
                        crate::actions::execute_action(clean);
                    }
                } else if cmd == "dismiss" {
                    ctrl_ipc.dismiss();
                } else if cmd == "quit" {
                    crate::ipc::remove_pet_pid();
                    let mut mem = crate::brain::CatMemory::load();
                    mem.mark_clean_exit();
                    std::process::exit(0);
                }
            }
        });

        let is_alive_destroy = Rc::clone(&is_alive);
        window.connect_destroy(move |_| {
            is_alive_destroy.set(false);
            crate::ipc::remove_pet_pid();
            let mut mem = crate::brain::CatMemory::load();
            mem.mark_clean_exit();
        });
    } else {
        let start_pos = anim_state.borrow().vector;
        root_fixed.put(&layout_container, start_pos.start_x, start_pos.start_y);

        let anim_tick = Rc::clone(&anim_state);
        let layout_tick = layout_container.clone();
        let win_weak = window.downgrade();
        let input_region_set = Cell::new(false);
        let bh_mask_tick = Rc::clone(&bubble_holder);

        root_fixed.add_tick_callback(move |fixed, frame_clock| {
            let now_us = frame_clock.frame_time();
            let (x, y, phase) = anim_tick.borrow_mut().tick(now_us);
            fixed.move_(&layout_tick, x, y);

            if phase == AnimPhase::Holding && !input_region_set.get() {
                if let Some(win) = win_weak.upgrade() {
                    if let Some(surface) = win.surface() {
                        let region = cairo::Region::create();
                        let _ = region.union_rectangle(&cat_rect);
                        let (cur_w, cur_h) = if let Some(w) = bh_mask_tick.borrow().as_ref() {
                            (w.width().max(bubble_w as i32), w.height().max((bubble_h + 14.0) as i32))
                        } else {
                            (bubble_w as i32, (bubble_h + 14.0) as i32)
                        };
                        let dynamic_bubble_rect = cairo::RectangleInt::new(
                            bubble_x as i32,
                            bubble_y as i32,
                            cur_w,
                            cur_h + 10,
                        );
                        let _ = region.union_rectangle(&dynamic_bubble_rect);
                        surface.set_input_region(Some(&region));
                        input_region_set.set(true);
                    }
                }
            } else if phase == AnimPhase::SlideOut && input_region_set.get() {
                if let Some(win) = win_weak.upgrade() {
                    if let Some(surface) = win.surface() {
                        let empty = cairo::Region::create();
                        surface.set_input_region(Some(&empty));
                        input_region_set.set(false);
                    }
                }
            } else if phase == AnimPhase::Done {
                if let Some(win) = win_weak.upgrade() {
                    win.close();
                }
                return glib::ControlFlow::Break;
            }

            glib::ControlFlow::Continue
        });
    }

    window.set_child(Some(&root_fixed));
    window.present();
    window
}

pub fn show_popups(
    app: &Application,
    config: &Config,
    repo_count: usize,
    aur_count: usize,
    flatpak_count: usize,
) {
    if config.visual.always_visible_pet_mode {
        std::mem::forget(app.hold());
    }

    if repo_count > 0 || aur_count > 0 || flatpak_count > 0 {
        play_meow(&config.audio);
    }

    if let Some(display) = Display::default() {
        let monitors = display.monitors();

        if config.general.show_on_all_monitors && !config.visual.always_visible_pet_mode {
            for i in 0..monitors.n_items() {
                if let Some(mon) = monitors.item(i).and_downcast::<Monitor>() {
                    spawn_popup_window(app, config, Some(&mon), repo_count, aur_count, flatpak_count);
                }
            }
        } else {
            let target_monitor =
                crate::ui::monitor::resolve_target_monitor(&display, &config.general.monitor);
            spawn_popup_window(
                app,
                config,
                target_monitor.as_ref(),
                repo_count,
                aur_count,
                flatpak_count,
            );
        }
    }
}
