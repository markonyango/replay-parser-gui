use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

pub const STATUS_EVENT: &str = "status-changed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IndicatorState {
    Ok,
    /// Working, but with reduced functionality.
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusIndicator {
    pub id: String,
    pub label: String,
    pub state: IndicatorState,
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct StatusSnapshot {
    pub text: String,
    pub indicators: Vec<StatusIndicator>,
}

/// Latest status, kept so the frontend can fetch it after it finishes loading.
#[derive(Default)]
pub struct StatusStore(Mutex<StatusSnapshot>);

impl StatusSnapshot {
    fn upsert(&mut self, indicator: StatusIndicator) {
        match self.indicators.iter_mut().find(|i| i.id == indicator.id) {
            Some(existing) => *existing = indicator,
            None => self.indicators.push(indicator),
        }
    }
}

fn update(handle: &AppHandle, change: impl FnOnce(&mut StatusSnapshot)) {
    let store = handle.state::<StatusStore>();
    let snapshot = {
        let mut snapshot = store.0.lock().unwrap_or_else(|e| e.into_inner());
        change(&mut snapshot);
        snapshot.clone()
    };

    if let Err(e) = handle.emit_to("main", STATUS_EVENT, snapshot) {
        tracing::warn!("Could not emit status update: {e}");
    }
}

pub fn set_text(handle: &AppHandle, text: impl Into<String>) {
    let text = text.into();
    update(handle, |s| s.text = text);
}

pub fn set_indicator(
    handle: &AppHandle,
    id: &str,
    label: &str,
    state: IndicatorState,
    tooltip: Option<String>,
) {
    let indicator = StatusIndicator {
        id: id.into(),
        label: label.into(),
        state,
        tooltip,
    };
    update(handle, |s| s.upsert(indicator));
}

#[tauri::command]
pub fn get_status(store: tauri::State<'_, StatusStore>) -> StatusSnapshot {
    store.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
}
