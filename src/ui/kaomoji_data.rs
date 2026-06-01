use gtk4::glib;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;

// --- GObject Definition ---

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct KaomojiObject {
        pub text: RefCell<String>,
        pub name: RefCell<String>,
        pub category: RefCell<KaomojiCategory>,
        pub keywords: RefCell<Vec<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KaomojiObject {
        const NAME: &'static str = "KaomojiObject";
        type Type = super::KaomojiObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for KaomojiObject {}
}

glib::wrapper! {
    pub struct KaomojiObject(ObjectSubclass<imp::KaomojiObject>);
}

impl KaomojiObject {
    pub fn new(
        text: String,
        name: String,
        category: KaomojiCategory,
        keywords: Vec<String>,
    ) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().text.borrow_mut() = text;
        *obj.imp().name.borrow_mut() = name;
        *obj.imp().category.borrow_mut() = category;
        *obj.imp().keywords.borrow_mut() = keywords;
        obj
    }

    pub fn text(&self) -> String {
        self.imp().text.borrow().clone()
    }

    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }

    pub fn category(&self) -> KaomojiCategory {
        *self.imp().category.borrow()
    }
}

// --- Data & Categories ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KaomojiCategory {
    #[default]
    Joy,
    Love,
    Embarrassment,
    Anger,
    Sorrow,
    Actions,
}

pub fn get_all_kaomojis() -> Vec<KaomojiObject> {
    let raw = vec![
        // JOY
        ("( ﾉ ﾟｰﾟ)ﾉ", "Hooray", KaomojiCategory::Joy),
        ("( 🌿☆‿‿☆)", "Star Eyes", KaomojiCategory::Joy),
        ("(* ^ ω ^)", "Happy", KaomojiCategory::Joy),
        ("(o^▽^o)", "Joy", KaomojiCategory::Joy),
        ("(´｡• ᵕ •｡`)", "Cute", KaomojiCategory::Joy),
        ("ヽ(・∀・)ﾉ", "Excited", KaomojiCategory::Joy),
        ("٩(◕‿◕｡)۶", "Cheer", KaomojiCategory::Joy),
        ("(o･ω･o)", "Bear shape", KaomojiCategory::Joy),
        // LOVE
        ("(♡˙︶˙♡)", "Love", KaomojiCategory::Love),
        ("( ˘ ³˘)♥", "Kiss", KaomojiCategory::Love),
        ("(´,,•ω•,,)♡", "Shy Love", KaomojiCategory::Love),
        ("❤ (ɔˆз(ˆ⌣ˆc)", "Hug", KaomojiCategory::Love),
        // EMBARRASSMENT
        ("(⁄ ⁄•⁄ω⁄•⁄ ⁄)", "Blush", KaomojiCategory::Embarrassment),
        ("(*/_＼)", "Hide", KaomojiCategory::Embarrassment),
        ("(◡‿◡ *)", "Shy", KaomojiCategory::Embarrassment),
        // ANGER
        ("(＃`Д´)", "Angry", KaomojiCategory::Anger),
        ("( ` ε ´ )", "Pout", KaomojiCategory::Anger),
        ("(╬ Ò﹏Ó)", "Rage", KaomojiCategory::Anger),
        ("凸(￣ヘ￣)", "Middle Finger", KaomojiCategory::Anger),
        // SORROW
        ("(╥_╥)", "Crying", KaomojiCategory::Sorrow),
        ("( o_-) /", "Comfort", KaomojiCategory::Sorrow),
        ("(｡•́︿•̀｡)", "Sad", KaomojiCategory::Sorrow),
        // ACTIONS / MEMES
        ("(╯°□°)╯︵ ┻━┻", "Table Flip", KaomojiCategory::Actions),
        ("(ノಠ益ಠ)ノ彡┻━┻", "Angry Flip", KaomojiCategory::Actions),
        ("┬─┬ノ( º _ ºノ)", "Table Set", KaomojiCategory::Actions),
        ("( ͡° ͜ʖ ͡°)", "Lenny Face", KaomojiCategory::Actions),
        ("¯\\_(ツ)_/¯", "Shrug", KaomojiCategory::Actions),
        ("ʕ•ᴥ•ʔ", "Bear", KaomojiCategory::Actions),
        ("uwu", "UWU", KaomojiCategory::Actions),
    ];

    raw.into_iter()
        .map(|(txt, name, cat)| {
            KaomojiObject::new(
                txt.to_string(),
                name.to_string(),
                cat,
                vec![name.to_lowercase()],
            )
        })
        .collect()
}
