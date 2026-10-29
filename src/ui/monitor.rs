use gtk4::gdk::prelude::*;
use gtk4::gdk::{Display, Monitor};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct KScreenOutput {
    name: String,
    #[serde(default)]
    primary: Option<bool>,
    #[serde(default)]
    priority: Option<u32>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    connected: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct KScreenDoctorData {
    #[serde(default)]
    outputs: Vec<KScreenOutput>,
}

/// Detects the primary monitor connector name on KDE Plasma via `kscreen-doctor -j`.
/// Supports:
/// 1. Output with `primary: true`
/// 2. Output with `priority: 1` (standard in KDE Plasma 6 Wayland)
/// 3. Output with lowest priority number among enabled & connected displays
pub fn detect_kde_primary_connector() -> Option<String> {
    let output = Command::new("kscreen-doctor")
        .arg("-j")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let data: KScreenDoctorData = serde_json::from_slice(&output.stdout).ok()?;

    // 1. Explicit primary: true
    if let Some(out) = data
        .outputs
        .iter()
        .find(|o| o.enabled.unwrap_or(true) && o.primary == Some(true))
    {
        return Some(out.name.clone());
    }

    // 2. Priority == 1 (KDE Plasma 6 Wayland convention)
    if let Some(out) = data
        .outputs
        .iter()
        .find(|o| o.enabled.unwrap_or(true) && o.priority == Some(1))
    {
        return Some(out.name.clone());
    }

    // 3. Lowest priority among enabled and connected
    let mut candidates: Vec<&KScreenOutput> = data
        .outputs
        .iter()
        .filter(|o| o.enabled.unwrap_or(true) && o.connected.unwrap_or(true))
        .collect();
    candidates.sort_by_key(|o| o.priority.unwrap_or(999));

    candidates.first().map(|o| o.name.clone())
}

#[derive(Debug, Clone)]
pub struct MonitorOption {
    pub id: String,    // e.g. "auto", "HDMI-A-1", "DP-2"
    pub label: String, // e.g. "Primary Display (Auto)", "HDMI-A-1 (Acer Technologies QG241Y)"
}

/// Lists all monitor options available for UI selection.
pub fn get_available_monitor_options(display: &Display) -> Vec<MonitorOption> {
    let mut options = vec![MonitorOption {
        id: "auto".to_string(),
        label: "Primary Display (Auto)".to_string(),
    }];

    let monitors = display.monitors();
    for i in 0..monitors.n_items() {
        if let Some(m) = monitors.item(i).and_downcast::<Monitor>() {
            if let Some(conn) = m.connector() {
                let conn_str = conn.to_string();
                let model = m.model().map(|d| d.to_string()).unwrap_or_default();
                let label = if model.is_empty() {
                    conn_str.clone()
                } else {
                    format!("{} ({})", conn_str, model)
                };
                options.push(MonitorOption {
                    id: conn_str,
                    label,
                });
            }
        }
    }

    options
}

/// Resolves the target `Monitor` to display notifications on, based on user configuration.
pub fn resolve_target_monitor(display: &Display, configured_monitor: &str) -> Option<Monitor> {
    let monitors = display.monitors();
    let n = monitors.n_items();
    if n == 0 {
        return None;
    }

    let mon_list: Vec<Monitor> = (0..n)
        .filter_map(|i| monitors.item(i).and_downcast::<Monitor>())
        .collect();

    // 1. If user configured an explicit monitor (not "auto" or empty)
    let configured = configured_monitor.trim();
    if !configured.is_empty() && !configured.eq_ignore_ascii_case("auto") {
        if let Some(target) = mon_list.iter().find(|m| {
            m.connector().as_deref() == Some(configured)
        }) {
            tracing::info!(
                "Using user-configured monitor: {} (model: {:?})",
                configured,
                target.model()
            );
            return Some(target.clone());
        }
        tracing::warn!(
            "Configured monitor '{}' not found among connected displays, falling back to auto-detection.",
            configured
        );
    }

    // 2. Query KDE Plasma primary display via kscreen-doctor
    if let Some(primary_conn) = detect_kde_primary_connector() {
        if let Some(target) = mon_list.iter().find(|m| {
            m.connector().as_deref() == Some(&primary_conn)
        }) {
            tracing::info!(
                "Resolved KDE primary monitor: {} (model: {:?})",
                primary_conn,
                target.model()
            );
            return Some(target.clone());
        }
    }

    // 3. Fallback: first monitor
    let fallback = mon_list.first().cloned();
    if let Some(ref m) = fallback {
        tracing::info!(
            "Falling back to default monitor: {:?} (model: {:?})",
            m.connector(),
            m.model()
        );
    }
    fallback
}
