use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

pub mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
    pub enum EmojiCategory {
        #[default]
        SmileysAndPeople,
        AnimalsAndNature,
        FoodAndDrink,
        Activities,
        TravelAndPlaces,
        Objects,
        Symbols,
        Flags,
        Recent, // Special
    }

    #[derive(Default)]
    pub struct EmojiObject {
        pub data: RefCell<String>,
        pub name: RefCell<String>,
        pub name_lower: RefCell<String>, // Cached
        pub category: RefCell<EmojiCategory>,
        pub keywords: RefCell<Vec<String>>,
        pub keywords_lower: RefCell<Vec<String>>, // Cached
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EmojiObject {
        const NAME: &'static str = "EmojiObject";
        type Type = super::EmojiObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for EmojiObject {}
}

glib::wrapper! {
    pub struct EmojiObject(ObjectSubclass<imp::EmojiObject>);
}

impl EmojiObject {
    pub fn new(emoji: String, name: String, category: imp::EmojiCategory, keywords: Vec<String>) -> Self {
        let obj = glib::Object::builder::<Self>().build();
        
        let name_lower = name.to_lowercase();
        let keywords_lower = keywords.iter().map(|k| k.to_lowercase()).collect();

        *obj.imp().data.borrow_mut() = emoji;
        *obj.imp().name.borrow_mut() = name;
        *obj.imp().name_lower.borrow_mut() = name_lower;
        *obj.imp().category.borrow_mut() = category;
        *obj.imp().keywords.borrow_mut() = keywords;
        *obj.imp().keywords_lower.borrow_mut() = keywords_lower;
        obj
    }

    pub fn emoji(&self) -> String {
        self.imp().data.borrow().clone()
    }

    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }
    
    // Optimized accessor
    pub fn name_lower(&self) -> String {
        self.imp().name_lower.borrow().clone()
    }
    
    pub fn category(&self) -> imp::EmojiCategory {
        *self.imp().category.borrow()
    }

    pub fn keywords(&self) -> Vec<String> {
        self.imp().keywords.borrow().clone()
    }
    
    // Optimized accessor
    pub fn keywords_lower(&self) -> Vec<String> {
        self.imp().keywords_lower.borrow().clone()
    }
}
