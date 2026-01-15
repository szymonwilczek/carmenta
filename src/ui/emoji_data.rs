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
        pub category: RefCell<EmojiCategory>, 
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
    pub fn new(emoji: String, name: String, category: imp::EmojiCategory) -> Self {
        let obj = glib::Object::builder::<Self>().build();
        *obj.imp().data.borrow_mut() = emoji;
        *obj.imp().name.borrow_mut() = name;
        *obj.imp().category.borrow_mut() = category;
        obj
    }

    pub fn emoji(&self) -> String {
        self.imp().data.borrow().clone()
    }

    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }
    
    pub fn category(&self) -> imp::EmojiCategory {
        *self.imp().category.borrow()
    }
}

