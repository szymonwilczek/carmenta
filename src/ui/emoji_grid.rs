use gtk4::prelude::*;
use gtk4::{
    gio, glib, GridView, SignalListItemFactory, SingleSelection, 
    PolicyType, ScrolledWindow, Box, Orientation, ToggleButton, 
    CustomFilter, FilterListModel
};
use super::emoji_data::EmojiObject;
use crate::ui::emoji_data::imp::EmojiCategory;
use crate::dbus::DBusClient;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_emoji_grid() -> Box {
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
    
    // Populate store (Async usually better, but keeping sync for MVP step)
    let mut all_emojis = Vec::new();
    for e in emojis::iter() {
        let cat = match e.group() {
            emojis::Group::SmileysAndEmotion => EmojiCategory::SmileysAndPeople,
            emojis::Group::PeopleAndBody => EmojiCategory::SmileysAndPeople,
            emojis::Group::AnimalsAndNature => EmojiCategory::AnimalsAndNature,
            emojis::Group::FoodAndDrink => EmojiCategory::FoodAndDrink,
            emojis::Group::Activities => EmojiCategory::Activities,
            emojis::Group::TravelAndPlaces => EmojiCategory::TravelAndPlaces,
            emojis::Group::Objects => EmojiCategory::Objects,
            emojis::Group::Symbols => EmojiCategory::Symbols,
            emojis::Group::Flags => EmojiCategory::Flags,
            _ => EmojiCategory::Symbols, // Fallback
        };

        all_emojis.push(EmojiObject::new(
            e.as_str().to_string(), 
            e.name().to_string(),
            cat
        ));
    }
    store.extend_from_slice(&all_emojis);

    // Current Category State
    // We use a shared state to update the filter
    let current_category = Rc::new(RefCell::new(EmojiCategory::SmileysAndPeople));
    
    let filter = CustomFilter::new(glib::clone!(@strong current_category => move |obj| {
        let emoji_obj = obj.downcast_ref::<EmojiObject>().unwrap();
        // Show if category matches
        // TODO: Handle 'Recent' handling later
        emoji_obj.category() == *current_category.borrow()
    }));

    let filter_model = FilterListModel::new(Some(store), Some(filter.clone()));
    let selection_model = SingleSelection::new(Some(filter_model));

    // 3. Category Buttons
    let categories = vec![
        ("🕙", EmojiCategory::Recent, "Recent"),
        ("🙂", EmojiCategory::SmileysAndPeople, "Smileys & People"),
        ("🐯", EmojiCategory::AnimalsAndNature, "Animals & Nature"),
        ("🍔", EmojiCategory::FoodAndDrink, "Food & Drink"),
        ("⚽", EmojiCategory::Activities, "Activities"),
        ("✈️", EmojiCategory::TravelAndPlaces, "Travel & Places"),
        ("💡", EmojiCategory::Objects, "Objects"),
        ("⁉️", EmojiCategory::Symbols, "Symbols"),
        ("🇺🇳", EmojiCategory::Flags, "Flags"),
    ];

    let group = gtk4::CheckButton::builder().build(); 
    
    // First button ref to set group
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
            btn.set_active(true); // Default active
        }

        // Logic
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

    // 4. Factory & Grid
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_factory, item| {
         let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
         let button = gtk4::Button::builder().css_classes(["emoji-btn", "flat"]).build();
         item.set_child(Some(&button));
         button.connect_clicked(move |btn| {
             let text = btn.label().unwrap_or_default().to_string();
             let ctx = glib::MainContext::default();
             ctx.spawn_local(async move {
                 DBusClient::insert_or_copy(&text).await;
             });
         });
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
        .max_columns(10)
        .min_columns(5)
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
