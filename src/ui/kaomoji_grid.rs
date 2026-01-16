use gtk4::prelude::*;
use gtk4::{
    gio, glib, GridView, SignalListItemFactory, SingleSelection, 
    PolicyType, ScrolledWindow, Box, Orientation, ToggleButton, 
    CustomFilter, FilterListModel
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::dbus::DBusClient;
use super::kaomoji_data::{KaomojiObject, KaomojiCategory, get_all_kaomojis};

pub fn create_kaomoji_grid(search_entry: &gtk4::SearchEntry) -> Box {
    let container = Box::new(Orientation::Horizontal, 0);
    container.set_css_classes(&["emoji-page"]); // Re-use styling

    // 1. Sidebar (Categories)
    let sidebar = Box::new(Orientation::Vertical, 6);
    sidebar.set_margin_start(6);
    sidebar.set_margin_end(6);
    sidebar.set_margin_top(6);
    sidebar.set_margin_bottom(6);

    // 2. Store
    let store = gio::ListStore::new::<KaomojiObject>();
    store.extend_from_slice(&get_all_kaomojis());

    // 3. Filter
    let current_category = Rc::new(RefCell::new(KaomojiCategory::Joy));
    let current_query = Rc::new(RefCell::new(String::new()));

    let filter = CustomFilter::new(glib::clone!(@strong current_category, @strong current_query => move |obj| {
        let kao = obj.downcast_ref::<KaomojiObject>().unwrap();
        let query = current_query.borrow();

        if !query.is_empty() {
            // Search name/text
            return kao.text().contains(query.as_str()) || 
                   kao.name().to_lowercase().contains(query.as_str());
        }
        
        // Show "Actions" as default for SafeMode/Actions combined? Or strict?
        // Let's be strict for now.
        kao.category() == *current_category.borrow()
    }));

    let filter_model = FilterListModel::new(Some(store), Some(filter.clone()));
    let selection_model = SingleSelection::new(Some(filter_model));

    // Connect Search with debounce (150ms)
    let debounce_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    search_entry.connect_search_changed(glib::clone!(@weak filter, @strong current_query, @strong debounce_source => move |entry| {
        // Cancel previous debounce timer if still pending
        if let Some(source_id) = debounce_source.borrow_mut().take() {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                source_id.remove();
            }));
        }
        
        let query = entry.text().to_string().to_lowercase();
        let current_query_clone = current_query.clone();
        let filter_weak = filter.downgrade();
        let debounce_source_clone = debounce_source.clone();
        
        let source_id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(150),
            move || {
                *debounce_source_clone.borrow_mut() = None;
                *current_query_clone.borrow_mut() = query;
                if let Some(f) = filter_weak.upgrade() {
                    f.changed(gtk4::FilterChange::Different);
                }
            }
        );
        *debounce_source.borrow_mut() = Some(source_id);
    }));

    // Buttons
    let categories = vec![
        ("😂", KaomojiCategory::Joy, "Joy"),
        ("❤", KaomojiCategory::Love, "Love"),
        ("😳", KaomojiCategory::Embarrassment, "Embarrassed"),
        ("💢", KaomojiCategory::Anger, "Anger"),
        ("😥", KaomojiCategory::Sorrow, "Sorrow"),
        ("┻━┻", KaomojiCategory::Actions, "Actions"),
    ];

    let mut first_btn: Option<ToggleButton> = None;
    for (icon, cat, tooltip) in categories {
        let btn = ToggleButton::builder()
            .label(icon)
            .tooltip_text(tooltip)
            .css_classes(["category-btn", "flat"])
            .build();

        if let Some(ref first) = first_btn {
            btn.set_group(Some(first));
        } else {
            first_btn = Some(btn.clone());
            btn.set_active(true);
        }

        let cat_val = cat;
        btn.connect_toggled(glib::clone!(@strong current_category, @weak filter => move |b| {
            if b.is_active() {
                *current_category.borrow_mut() = cat_val;
                filter.changed(gtk4::FilterChange::Different);
            }
        }));
        sidebar.append(&btn);
    }
    container.append(&sidebar);

    // 4. Grid Factory
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_factory, item| {
         let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
         let button = gtk4::Button::builder().css_classes(["kaomoji-btn", "flat"]).build(); // New class
         item.set_child(Some(&button));
         button.connect_clicked(move |btn| {
             let text = btn.label().unwrap_or_default().to_string();
             
             // History + Insertion Logic
             crate::app::mark_inserting();
             crate::history::add_recent(text.clone());
             
             DBusClient::insert_or_copy(&text);
         });
    });

    factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let button = item.child().unwrap().downcast::<gtk4::Button>().unwrap();
        let entry = item.item().unwrap().downcast::<KaomojiObject>().unwrap();
        button.set_label(&entry.text());
        button.set_tooltip_text(Some(&entry.name()));
    });

    let grid_view = GridView::builder()
        .model(&selection_model)
        .factory(&factory)
        .max_columns(3) // Less columns, wider items
        .min_columns(2)
        .enable_rubberband(false)
        .build();

    let scrolled = ScrolledWindow::builder()
        .child(&grid_view)
        .hscrollbar_policy(PolicyType::Never)
        .hexpand(true)
        .vexpand(true)
        .build();

    container.append(&scrolled);
    
    container
}
