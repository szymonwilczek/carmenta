use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;
use unicode_blocks as ub;

// --- GObject Definition ---

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct SymbolObject {
        pub char: RefCell<String>,
        pub name: RefCell<String>, // block name
        pub category: RefCell<SymbolCategory>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SymbolObject {
        const NAME: &'static str = "SymbolObject";
        type Type = super::SymbolObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for SymbolObject {}
}

glib::wrapper! {
    pub struct SymbolObject(ObjectSubclass<imp::SymbolObject>);
}

impl SymbolObject {
    pub fn new(c: char, block_name: &str, category: SymbolCategory) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().char.borrow_mut() = c.to_string();
        *obj.imp().name.borrow_mut() = format!("{} ({})", c, block_name); // placeholder name
        *obj.imp().category.borrow_mut() = category;
        obj
    }

    pub fn char(&self) -> String {
        self.imp().char.borrow().clone()
    }
    
    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }

    pub fn category(&self) -> SymbolCategory {
        *self.imp().category.borrow()
    }
}

// --- Data & Categories ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SymbolCategory {
    #[default]
    Arrows,
    Math,
    Currency,
    Tech,
    BoxDrawing,
    Misc,
}

pub fn get_symbols() -> Vec<SymbolObject> {
    let mut symbols = Vec::new();

    let blocks_to_scan = vec![
        (ub::ARROWS, SymbolCategory::Arrows, "Arrows"),
        (ub::SUPPLEMENTAL_ARROWS_A, SymbolCategory::Arrows, "Supp. Arrows A"),
        (ub::SUPPLEMENTAL_ARROWS_B, SymbolCategory::Arrows, "Supp. Arrows B"),
        
        (ub::MATHEMATICAL_OPERATORS, SymbolCategory::Math, "Math"),
        (ub::SUPPLEMENTAL_MATHEMATICAL_OPERATORS, SymbolCategory::Math, "Supp. Math"),
        
        (ub::CURRENCY_SYMBOLS, SymbolCategory::Currency, "Currency"),
        
        (ub::MISCELLANEOUS_TECHNICAL, SymbolCategory::Tech, "Technical"),
        (ub::CONTROL_PICTURES, SymbolCategory::Tech, "Control"),
        
        (ub::BOX_DRAWING, SymbolCategory::BoxDrawing, "Box Drawing"),
        (ub::BLOCK_ELEMENTS, SymbolCategory::BoxDrawing, "Block"),
        
        (ub::MISCELLANEOUS_SYMBOLS, SymbolCategory::Misc, "Misc"),
        (ub::DINGBATS, SymbolCategory::Misc, "Dingbats"),
    ];

    for (block, cat, label) in blocks_to_scan {
        let start = block.start() as u32; 
        let end = block.end() as u32;

        for code in start..=end {
            if let Some(c) = std::char::from_u32(code) {
                 symbols.push(SymbolObject::new(c, label, cat));
            }
        }
    }

    symbols
}
