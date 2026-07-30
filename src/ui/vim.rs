//! Optional Vim-style keyboard navigation (`--vim`).
//!
//! * `Alt` + `hjkl` moves *between* zones (never between items),
//!   so the search box and the menu button are always one chord away.
//! * plain `hjkl` moves *within* the focused zone, and is swallowed there
//!   so it never leaks into the search entry as typed text.
//!
//! Nothing here runs unless `--vim` was passed: the controller is always
//! installed (the window is long-lived and re-used across invocations with
//! different flags) but returns immediately when the mode is off.

use gtk4::gdk::{Key, ModifierType};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Button, DirectionType, GridView, ToggleButton, Widget};
use libadwaita::{ApplicationWindow, ViewStack};

/// CSS class marking page's category sidebar.
/// Pages are built independently, so the navigator locates the sidebar
/// of whichever page is visible by class rather than holding references
/// to all of them.
pub const SIDEBAR_CLASS: &str = "category-sidebar";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Zone {
    Input,
    Menu,
    Sidebar,
    Grid,
    Other,
}

/// Navigable widgets of the currently visible stack page.
/// Both are optional: GIF and search-results pages have no category sidebar.
struct PageParts {
    sidebar: Option<Widget>,
    grid: Option<GridView>,
}

/// Install the Vim navigation controller on the window.
pub fn install(
    window: &ApplicationWindow,
    search_entry: &gtk4::SearchEntry,
    menu_button: &gtk4::MenuButton,
    stack: &ViewStack,
) {
    let controller = gtk4::EventControllerKey::new();
    // capture so grid/sidebar movement keys are consumed before
    // the focused widget (or the search entry) ever sees them
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);

    let search_entry = search_entry.clone();
    let menu_button = menu_button.clone();
    let stack = stack.clone();
    let window_weak = window.downgrade();

    controller.connect_key_pressed(move |_, key, _, modifier| {
        if !crate::vim_enabled() {
            return glib::Propagation::Proceed;
        }
        let Some(window) = window_weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        handle(&window, &search_entry, &menu_button, &stack, key, modifier)
    });

    window.add_controller(controller);
}

fn handle(
    window: &ApplicationWindow,
    search_entry: &gtk4::SearchEntry,
    menu_button: &gtk4::MenuButton,
    stack: &ViewStack,
    key: Key,
    modifier: ModifierType,
) -> glib::Propagation {
    let focus = gtk4::prelude::RootExt::focus(window);
    let parts = page_parts(stack);
    let zone = focus
        .as_ref()
        .map(|f| zone_of(f, search_entry, menu_button, &parts))
        .unwrap_or(Zone::Other);

    if modifier.contains(ModifierType::ALT_MASK) {
        if !is_motion(key) {
            return glib::Propagation::Proceed;
        }
        match (zone, key) {
            // top bar: search entry <-> menu ("About") button
            (Zone::Input, Key::l) => {
                menu_button.grab_focus();
            }
            (Zone::Menu, Key::h) => {
                search_entry.grab_focus();
            }
            // down from the top bar lands in the open category's items
            (Zone::Input | Zone::Menu | Zone::Other, Key::j) => {
                focus_grid(&parts);
            }
            // up from anywhere in the content area returns to the search box
            (Zone::Grid | Zone::Sidebar, Key::k) => {
                search_entry.grab_focus();
            }
            (Zone::Grid, Key::h) => {
                focus_sidebar(&parts);
            }
            (Zone::Sidebar, Key::l) => {
                focus_grid(&parts);
            }
            _ => {}
        }
        // swallow every Alt+hjkl, including the no-op combinations,
        // so unhandled chord can never end up as text in the search entry
        return glib::Propagation::Stop;
    }

    match zone {
        Zone::Grid => {
            let Some(grid) = parts.grid.as_ref() else {
                return glib::Propagation::Proceed;
            };
            match key {
                Key::h => grid.child_focus(DirectionType::Left),
                Key::j => grid.child_focus(DirectionType::Down),
                Key::k => grid.child_focus(DirectionType::Up),
                Key::l => grid.child_focus(DirectionType::Right),
                Key::Return | Key::KP_Enter | Key::space => {
                    activate_focused(focus.as_ref())
                }
                _ => return glib::Propagation::Proceed,
            };
            glib::Propagation::Stop
        }
        Zone::Sidebar => {
            match key {
                Key::j => move_category(&parts, DirectionType::Down),
                Key::k => move_category(&parts, DirectionType::Up),
                Key::Return | Key::KP_Enter | Key::space => activate_focused(focus.as_ref()),
                _ => return glib::Propagation::Proceed,
            };
            glib::Propagation::Stop
        }
        // in the search entry hjkl are ordinary characters again
        _ => glib::Propagation::Proceed,
    }
}

fn is_motion(key: Key) -> bool {
    matches!(key, Key::h | Key::j | Key::k | Key::l)
}

fn page_parts(stack: &ViewStack) -> PageParts {
    let Some(page) = stack.visible_child() else {
        return PageParts {
            sidebar: None,
            grid: None,
        };
    };
    PageParts {
        sidebar: find_descendant(&page, &|w| w.has_css_class(SIDEBAR_CLASS)),
        grid: find_descendant(&page, &|w| w.is::<GridView>())
            .and_then(|w| w.downcast::<GridView>().ok()),
    }
}

fn zone_of(
    focus: &Widget,
    search_entry: &gtk4::SearchEntry,
    menu_button: &gtk4::MenuButton,
    parts: &PageParts,
) -> Zone {
    if within(focus, search_entry.upcast_ref()) {
        return Zone::Input;
    }
    if within(focus, menu_button.upcast_ref()) {
        return Zone::Menu;
    }
    if parts.sidebar.as_ref().is_some_and(|s| within(focus, s)) {
        return Zone::Sidebar;
    }
    if parts
        .grid
        .as_ref()
        .is_some_and(|g| within(focus, g.upcast_ref()))
    {
        return Zone::Grid;
    }
    Zone::Other
}

fn within(focus: &Widget, ancestor: &Widget) -> bool {
    focus == ancestor || focus.is_ancestor(ancestor)
}

/// Focus the item grid, restoring whichever item was focused there last.
fn focus_grid(parts: &PageParts) -> bool {
    let Some(grid) = parts.grid.as_ref() else {
        return false;
    };
    grid.grab_focus() || grid.child_focus(DirectionType::TabForward)
}

/// Focus the category sidebar on the category that is currently open.
fn focus_sidebar(parts: &PageParts) -> bool {
    let Some(sidebar) = parts.sidebar.as_ref() else {
        return false;
    };
    if let Some(active) = find_descendant(sidebar, &|w| {
        w.downcast_ref::<ToggleButton>()
            .is_some_and(|b| b.is_active())
    }) {
        if active.grab_focus() {
            return true;
        }
    }
    sidebar.child_focus(DirectionType::TabForward)
}

/// Move the category cursor and open the category it lands on,
/// so the grid always shows what the sidebar cursor points at.
fn move_category(parts: &PageParts, direction: DirectionType) -> bool {
    let Some(sidebar) = parts.sidebar.as_ref() else {
        return false;
    };
    if !sidebar.child_focus(direction) {
        return false;
    }
    let Some(root) = sidebar.root() else {
        return false;
    };
    if let Some(focus) = gtk4::prelude::RootExt::focus(&root) {
        if let Some(toggle) = focus.downcast_ref::<ToggleButton>() {
            toggle.set_active(true);
        }
    }
    true
}

/// Activate the button under the cursor.
/// Focused widget is normally the item's button itself,
/// but may be the list item wrapping it.
fn activate_focused(focus: Option<&Widget>) -> bool {
    let Some(focus) = focus else {
        return false;
    };
    if focus.is::<Button>() {
        return focus.activate();
    }
    match find_descendant(focus, &|w| w.is::<Button>()) {
        Some(button) => button.activate(),
        None => false,
    }
}

/// First widget in `root`'s subtree (depth-first) matching `pred`.
fn find_descendant(root: &Widget, pred: &dyn Fn(&Widget) -> bool) -> Option<Widget> {
    let mut child = root.first_child();
    while let Some(current) = child {
        if pred(&current) {
            return Some(current);
        }
        if let Some(found) = find_descendant(&current, pred) {
            return Some(found);
        }
        child = current.next_sibling();
    }
    None
}
