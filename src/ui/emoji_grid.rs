use gtk4::prelude::*;
use gtk4::{
    gio, glib, GridView, SignalListItemFactory, SingleSelection, 
    PolicyType, ScrolledWindow, Box, Orientation, ToggleButton, 
    CustomFilter, FilterListModel, Popover
};
use super::emoji_data::{EmojiCategory, EmojiObject, get_all_emojis};
use crate::dbus::DBusClient;
use std::cell::RefCell;
use std::rc::Rc;

// helper function: Insert text & manage history/focus
fn insert_helper(text: String) {
     crate::app::mark_inserting();
     crate::history::add_recent(text.clone());
     
     DBusClient::insert_or_copy(&text);
}

pub fn create_emoji_grid(search_entry: &gtk4::SearchEntry) -> Box {
    // Top container: Categories + Grid
    let container = Box::new(Orientation::Horizontal, 0);
    container.set_css_classes(&["emoji-page"]);

    // 1. Sidebar (Categories)
    let sidebar = Box::new(Orientation::Vertical, 6);
    sidebar.set_margin_start(6);
    sidebar.set_margin_end(6);
    sidebar.set_margin_top(6);
    sidebar.set_margin_bottom(6);
    
    // 2. Data Store & Filter
    let store = gio::ListStore::new::<EmojiObject>();
    
    // Populate store with all emojis first
    let all_emojis = get_all_emojis();
    store.extend_from_slice(&all_emojis);
    
    // helper function to rebuild Recent items in store
    fn rebuild_recent(store: &gio::ListStore) {
        // Remove existing Recent items (at the beginning)
        while store.n_items() > 0 {
            if let Some(obj) = store.item(0) {
                if let Some(emoji_obj) = obj.downcast_ref::<EmojiObject>() {
                    if emoji_obj.category() == EmojiCategory::Recent {
                        store.remove(0);
                        continue;
                    }
                }
            }
            break;
        }
        
        // Add current Recent items at the beginning
        let recent = crate::history::get_recent();
        for (i, r) in recent.iter().enumerate() {
            if let Some(e) = emojis::get(r) {
                let name = e.name().to_string();
                let mut keywords = vec![name.clone()];
                if let Some(short) = e.shortcode() {
                    keywords.push(short.to_string());
                }
                store.insert(i as u32, &EmojiObject::new(
                    r.clone(), 
                    name, 
                    EmojiCategory::Recent,
                    keywords
                ));
            }
        }
    }
    
    // Initial population of Recent
    rebuild_recent(&store);
    
    // Register callback to refresh Recent when history changes
    let store_weak = store.downgrade();
    crate::history::on_history_changed(move || {
        if let Some(store) = store_weak.upgrade() {
            rebuild_recent(&store);
        }
    });

    // Filter Logic
    let current_category = Rc::new(RefCell::new(EmojiCategory::SmileysAndPeople));
    let current_query = Rc::new(RefCell::new(String::new()));

    let filter = CustomFilter::new(glib::clone!(@strong current_category, @strong current_query => move |obj| {
        let emoji_obj = obj.downcast_ref::<EmojiObject>().unwrap();
        let query = current_query.borrow();
        
        // 1. Search filter
        if !query.is_empty() {
            // skip Recent category during search to avoid duplicates
            if emoji_obj.category() == EmojiCategory::Recent {
                return false;
            }
            
            let keywords = emoji_obj.keywords_lower();
            for k in keywords {
                if k.contains(query.as_str()) {
                    return true;
                }
            }
            return false;
        }

        // 2. Category filter
        emoji_obj.category() == *current_category.borrow()
    }));

    let filter_model = FilterListModel::new(Some(store), Some(filter.clone()));
    let selection_model = SingleSelection::new(Some(filter_model));

    // Connect Search Entry with debounce (150ms)
    let debounce_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    search_entry.connect_search_changed(glib::clone!(@weak filter, @strong current_query, @strong debounce_source => move |entry| {
        // Cancel previous debounce timer if still pending
        if let Some(source_id) = debounce_source.borrow_mut().take() {
            // Use try pattern - source may have already fired and been auto-removed
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                source_id.remove();
            }));
        }
        
        let query = entry.text().to_string().to_lowercase();
        let current_query_clone = current_query.clone();
        let filter_weak = filter.downgrade();
        let debounce_source_clone = debounce_source.clone();
        
        // Start new debounce timer
        let source_id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(150),
            move || {
                // Clear the source reference since timer fired (source auto-removed)
                *debounce_source_clone.borrow_mut() = None;
                *current_query_clone.borrow_mut() = query;
                if let Some(f) = filter_weak.upgrade() {
                    f.changed(gtk4::FilterChange::Different);
                }
            }
        );
        *debounce_source.borrow_mut() = Some(source_id);
    }));

    // 3. Category Buttons (Sidebar)
    let categories = [
        ("🕙", EmojiCategory::Recent),
        ("🙂", EmojiCategory::SmileysAndPeople),
        ("🐻", EmojiCategory::AnimalsAndNature),
        ("🍔", EmojiCategory::FoodAndDrink),
        ("⚽", EmojiCategory::Activities),
        ("✈️", EmojiCategory::TravelAndPlaces),
        ("💡", EmojiCategory::Objects),
        ("🔣", EmojiCategory::Symbols),
        ("🚩", EmojiCategory::Flags),
    ];

    let mut first_btn = None;

    for (icon, cat_val) in categories {
        let btn = ToggleButton::builder()
            .label(icon)
            .css_classes(["category-btn", "flat"])
            .build();
        
        if first_btn.is_none() {
            first_btn = Some(btn.clone());
        } else if let Some(ref first) = first_btn {
            btn.set_group(Some(first));
        }

        if cat_val == EmojiCategory::SmileysAndPeople {
             btn.set_active(true);
        } 

        btn.connect_toggled(glib::clone!(@strong current_category, @weak filter => move |b| {
            if b.is_active() {
                *current_category.borrow_mut() = cat_val;
                filter.changed(gtk4::FilterChange::Different);
            }
        }));
        
        sidebar.append(&btn);
    }
    
    container.append(&sidebar);

    // 4. Factory & Grid
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_factory, item| {
         let button = gtk4::Button::builder().css_classes(["emoji-btn", "flat"]).build();
         item.set_child(Some(&button));
         
         // Left Click (Primary)
         button.connect_clicked(move |btn| {
             let text = btn.label().unwrap_or_default().to_string();
             insert_helper(text);
         });

         // Right Click (Secondary) - Skin Tones
         let gesture = gtk4::GestureClick::new();
         gesture.set_button(3); // Right click
         
         let button_weak = button.downgrade();
         gesture.connect_pressed(move |_gesture, _, _, _| {
             let btn = match button_weak.upgrade() {
                 Some(b) => b,
                 None => return,
             };
             let base_emoji = btn.label().unwrap_or_default().to_string();
             
             if let Some(emoji_data) = emojis::get(&base_emoji) {
                 if let Some(variants) = emoji_data.skin_tones() {
                     // Create Popover
                     let popover = Popover::builder().child(&Box::new(Orientation::Horizontal, 5)).build();
                     let container = popover.child().unwrap().downcast::<Box>().unwrap();
                     container.set_margin_top(5);
                     container.set_margin_bottom(5);
                     container.set_margin_start(5);
                     container.set_margin_end(5);

                     // Add variants
                     for variant in variants {
                         let v_btn = gtk4::Button::builder()
                            .label(variant.as_str())
                            .css_classes(["emoji-btn-small", "flat"])
                            .build();
                         
                         let v_text = variant.as_str().to_string();
                         let pop_clone = popover.clone();
                         v_btn.connect_clicked(move |_| {
                             insert_helper(v_text.clone());
                             pop_clone.popdown();
                         });
                         container.append(&v_btn);
                     }
                     
                     popover.set_parent(&btn);
                     popover.popup();
                 }
             }
         });
         button.add_controller(gesture);
    });

    factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let button = item.child().unwrap().downcast::<gtk4::Button>().unwrap();
        let entry = item.item().unwrap().downcast::<EmojiObject>().unwrap();
        button.set_label(&entry.emoji());
        button.set_tooltip_text(Some(&entry.name()));
    });

    let grid_view = GridView::builder()
        .model(&selection_model)
        .factory(&factory)
        .max_columns(8)
        .min_columns(5)
        .build();

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&grid_view)
        .hexpand(true)
        .vexpand(true)
        .build();

    container.append(&scrolled_window);
    container
}
