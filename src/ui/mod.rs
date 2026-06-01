pub mod emoji_data;
pub mod emoji_grid;
pub mod gif_data;
pub mod gif_grid;
pub mod kaomoji_data;
pub mod kaomoji_grid;
pub mod symbols_data;
pub mod symbols_grid;

use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Insert text into the focused app (via the extension, or clipboard fallback),
/// record it in history, and close afterwards when requested.
pub fn insert_text(text: String, close_after: bool) {
    crate::app::mark_inserting();
    crate::history::add_recent(text.clone());
    crate::dbus::DBusClient::insert_or_copy(&text, close_after);
}

/// Wire up "Enter = commit the first visible item and close" for a filtered
/// grid: flush the debounce, take item 0, extract its text via `extract`, and
/// insert it. Collapses the otherwise-identical handler in every grid.
pub fn on_search_enter_commit<T, F>(
    search_entry: &gtk4::SearchEntry,
    container: &gtk4::Box,
    selection_model: &gtk4::SingleSelection,
    debounce_source: &Rc<RefCell<Option<glib::SourceId>>>,
    current_query: &Rc<RefCell<String>>,
    filter: &gtk4::CustomFilter,
    extract: F,
) where
    T: glib::prelude::IsA<glib::Object>,
    F: Fn(&T) -> String + 'static,
{
    let entry = search_entry.clone();
    let selection_model = selection_model.clone();
    let debounce_source = debounce_source.clone();
    let current_query = current_query.clone();
    let filter = filter.clone();
    on_search_enter(search_entry, container, move || {
        flush_debounce(&debounce_source, &entry, &current_query, &filter);
        if let Some(obj) = selection_model.item(0) {
            if let Ok(item) = obj.downcast::<T>() {
                insert_text(extract(&item), true);
            }
        }
    });
}

/// Apply the search box's current text to a grid's filter immediately,
/// cancelling any pending debounce timer. Used when the user commits with
/// Enter so the first item reflects exactly what was typed.
pub fn flush_debounce(
    debounce_source: &Rc<RefCell<Option<glib::SourceId>>>,
    entry: &gtk4::SearchEntry,
    current_query: &Rc<RefCell<String>>,
    filter: &gtk4::CustomFilter,
) {
    if let Some(source_id) = debounce_source.borrow_mut().take() {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            source_id.remove();
        }));
    }
    *current_query.borrow_mut() = entry.text().to_string().to_lowercase();
    filter.changed(gtk4::FilterChange::Different);
}

/// Wire up "Enter in the search box = commit on the visible page".
///
/// Uses an `EventControllerKey` in the Capture phase so the entry's internal
/// text widget doesn't consume Return first. `action` runs only when this
/// grid's `container` is the mapped (visible) stack page, so every grid can
/// register its own handler against the shared search entry without clashing.
pub fn on_search_enter<F>(search_entry: &gtk4::SearchEntry, container: &gtk4::Box, action: F)
where
    F: Fn() + 'static,
{
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let container_weak = container.downgrade();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key != gtk4::gdk::Key::Return && key != gtk4::gdk::Key::KP_Enter {
            return glib::Propagation::Proceed;
        }
        // Only the currently visible page should act.
        match container_weak.upgrade() {
            Some(c) if c.is_mapped() => {}
            _ => return glib::Propagation::Proceed,
        }
        action();
        glib::Propagation::Stop
    });
    search_entry.add_controller(key_controller);
}
