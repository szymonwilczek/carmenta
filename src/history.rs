use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::cell::RefCell;
use gtk4::glib;

const MAX_HISTORY: usize = 50;
const HISTORY_FILE: &str = "history.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct History {
    pub recent: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        Self { recent: Vec::new() }
    }

    fn get_path() -> PathBuf {
        let mut path = glib::user_data_dir(); // ~/.local/share
        path.push("carmenta");
        std::fs::create_dir_all(&path).ok();
        path.push(HISTORY_FILE);
        path
    }

    pub fn load() -> Self {
        let path = Self::get_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(history) = serde_json::from_str(&content) {
                return history;
            }
        }
        Self::new()
    }

    pub fn save(&self) {
        let path = Self::get_path();
        if let Ok(json) = serde_json::to_string(self) {
            if let Err(e) = fs::write(&path, json) {
                eprintln!("Failed to save history: {}", e);
            }
        }
    }

    pub fn add(&mut self, emoji: String) {
        // Remove existing to bubble to top
        if let Some(pos) = self.recent.iter().position(|x| *x == emoji) {
            self.recent.remove(pos);
        }
        // Insert at beginning
        self.recent.insert(0, emoji);
        // Trim
        if self.recent.len() > MAX_HISTORY {
            self.recent.truncate(MAX_HISTORY);
        }
        self.save();
    }
}

// Global history instance
thread_local! {
    pub static GLOBAL_HISTORY: RefCell<History> = RefCell::new(History::load());
    // Callbacks to notify UI when history changes
    static HISTORY_CALLBACKS: RefCell<Vec<Box<dyn Fn()>>> = RefCell::new(Vec::new());
}

pub fn add_recent(emoji: String) {
    GLOBAL_HISTORY.with(|h| {
        h.borrow_mut().add(emoji);
    });
    // Notify all registered callbacks
    notify_history_changed();
}

pub fn get_recent() -> Vec<String> {
    GLOBAL_HISTORY.with(|h| {
        h.borrow().recent.clone()
    })
}

/// Register a callback to be called when history changes
pub fn on_history_changed<F: Fn() + 'static>(callback: F) {
    HISTORY_CALLBACKS.with(|callbacks| {
        callbacks.borrow_mut().push(Box::new(callback));
    });
}

/// Notify all registered callbacks that history has changed
fn notify_history_changed() {
    HISTORY_CALLBACKS.with(|callbacks| {
        for callback in callbacks.borrow().iter() {
            callback();
        }
    });
}
