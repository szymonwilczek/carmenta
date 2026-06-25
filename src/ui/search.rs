use super::emoji_data::get_all_emojis;
use super::kaomoji_data::get_all_kaomojis;
use super::symbols_data::get_symbols;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{
    gio, Box, GridView, Orientation, PolicyType, ScrolledWindow, SignalListItemFactory,
    SingleSelection,
};
use libadwaita::ViewStack;
use std::cell::RefCell;
use std::rc::Rc;

pub const RESULTS_PAGE: &str = "results";

const MAX_RESULTS: usize = 300;

/// Which browsing tab a result originated from.
/// Used both to pick the right styling and to prioritise the active tab in the ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTab {
    Emoji,
    Kaomoji,
    Symbols,
}

impl SourceTab {
    /// `ViewStack` child name of the corresponding browsing page
    fn page_name(self) -> &'static str {
        match self {
            SourceTab::Emoji => "emoji",
            SourceTab::Kaomoji => "kaomoji",
            SourceTab::Symbols => "symbols",
        }
    }
}

mod imp {
    use super::*;

    pub struct SearchResultObject {
        pub display: RefCell<String>, // text inserted and shown on the button
        pub name: RefCell<String>,    // tooltip
        pub source: RefCell<SourceTab>,
    }

    impl Default for SearchResultObject {
        fn default() -> Self {
            Self {
                display: RefCell::new(String::new()),
                name: RefCell::new(String::new()),
                source: RefCell::new(SourceTab::Emoji),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchResultObject {
        const NAME: &'static str = "SearchResultObject";
        type Type = super::SearchResultObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for SearchResultObject {}
}

glib::wrapper! {
    pub struct SearchResultObject(ObjectSubclass<imp::SearchResultObject>);
}

impl SearchResultObject {
    fn new(display: &str, name: &str, source: SourceTab) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().display.borrow_mut() = display.to_string();
        *obj.imp().name.borrow_mut() = name.to_string();
        *obj.imp().source.borrow_mut() = source;
        obj
    }

    fn display(&self) -> String {
        self.imp().display.borrow().clone()
    }

    fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }

    fn source(&self) -> SourceTab {
        *self.imp().source.borrow()
    }
}

/// Single searchable entry, flattened from one of the browsing tabs
struct Candidate {
    display: String,
    name: String,
    haystack: String, // lowercased text fuzzy-matched against the query
    source: SourceTab,
}

/// Flatten emoji, kaomoji and symbols into one searchable corpus.
/// GIFs are intentionally excluded (see issue #13)
fn build_corpus() -> Vec<Candidate> {
    let mut corpus = Vec::new();

    for e in get_all_emojis() {
        corpus.push(Candidate {
            display: e.emoji(),
            name: e.name(),
            haystack: e.keywords_lower().join(" "),
            source: SourceTab::Emoji,
        });
    }

    for k in get_all_kaomojis() {
        let name = k.name();
        corpus.push(Candidate {
            haystack: format!("{} {}", name, k.text()).to_lowercase(),
            display: k.text(),
            name,
            source: SourceTab::Kaomoji,
        });
    }

    for s in get_symbols() {
        let name = s.name();
        corpus.push(Candidate {
            haystack: name.to_lowercase(),
            display: s.char(),
            name,
            source: SourceTab::Symbols,
        });
    }

    corpus
}

/// Score a candidate against (possibly multi-word) query.
///
/// Query is tokenised on whitespace and every token is fuzzy-matched independently;
/// all tokens must match and the score is their sum.
/// This makes matching word-order independent, so `arrow right` finds `right arrow`
/// even though that is not a single fuzzy subsequence.
fn score(matcher: &SkimMatcherV2, haystack: &str, tokens: &[&str]) -> Option<i64> {
    let mut total = 0;
    for token in tokens {
        total += matcher.fuzzy_match(haystack, token)?;
    }
    Some(total)
}

/// Build the merged cross-tab search results page and wire it to the shared
/// search entry.
/// While the query is empty the page restores the last browsing tab;
/// once the user types, results from emoji + kaomoji + symbols are ranked
/// (active tab first, then by fuzzy score) and shown here.
pub fn create_search_results_grid(
    search_entry: &gtk4::SearchEntry,
    stack: &ViewStack,
    last_browse: Rc<RefCell<String>>,
) -> Box {
    let container = Box::new(Orientation::Horizontal, 0);
    container.set_css_classes(&["emoji-page"]);
    // browsing pages get their left inset from the category sidebar;
    // results page has none, so add matching margins to avoid hugging the edge
    container.set_margin_start(6);
    container.set_margin_top(6);
    container.set_margin_bottom(6);

    let corpus = Rc::new(build_corpus());
    let store = gio::ListStore::new::<SearchResultObject>();

    let selection_model = SingleSelection::new(Some(store.clone()));

    // button per result, styled per source so wide kaomoji keep room
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let button = gtk4::Button::builder().css_classes(["emoji-btn", "flat"]).build();
        item.set_child(Some(&button));
        button.connect_clicked(move |btn| {
            let text = btn.label().unwrap_or_default().to_string();
            super::insert_text(text, crate::close_on_select());
        });
    });
    factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let button = item.child().unwrap().downcast::<gtk4::Button>().unwrap();
        let entry = item.item().unwrap().downcast::<SearchResultObject>().unwrap();
        button.set_label(&entry.display());
        button.set_tooltip_text(Some(&entry.name()));
        let glyph_class = match entry.source() {
            SourceTab::Kaomoji => "kaomoji-btn",
            _ => "emoji-btn",
        };
        button.set_css_classes(&[glyph_class, "flat"]);
    });

    let grid_view = GridView::builder()
        .model(&selection_model)
        .factory(&factory)
        .max_columns(8)
        .min_columns(3)
        .enable_rubberband(false)
        .build();

    let scrolled = ScrolledWindow::builder()
        .child(&grid_view)
        .hscrollbar_policy(PolicyType::Never)
        .hexpand(true)
        .vexpand(true)
        .build();
    container.append(&scrolled);

    // recompute and render the ranked results for `query`
    // (already lowercased)
    let matcher = Rc::new(SkimMatcherV2::default());
    let populate = {
        let store = store.clone();
        let corpus = corpus.clone();
        let matcher = matcher.clone();
        let last_browse = last_browse.clone();
        move |query: &str| {
            let tokens: Vec<&str> = query.split_whitespace().collect();
            let active = last_browse.borrow().clone();

            let mut scored: Vec<(i64, &Candidate)> = corpus
                .iter()
                .filter_map(|c| score(&matcher, &c.haystack, &tokens).map(|s| (s, c)))
                .collect();

            // Best match first (highest score)
            //
            // active tab only breaks ties, so it gets a nudge without burying
            // stronger match from another tab
            scored.sort_by(|a, b| {
                b.0.cmp(&a.0).then_with(|| {
                    let a_active = a.1.source.page_name() == active;
                    let b_active = b.1.source.page_name() == active;
                    b_active.cmp(&a_active)
                })
            });

            let items: Vec<SearchResultObject> = scored
                .into_iter()
                .take(MAX_RESULTS)
                .map(|(_, c)| SearchResultObject::new(&c.display, &c.name, c.source))
                .collect();

            store.remove_all();
            store.extend_from_slice(&items);
        }
    };

    // Debounced search:
    // switch to this page (and rank) while typing,
    // restore the last browsing tab when the query is cleared
    let debounce_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    search_entry.connect_search_changed(glib::clone!(
        #[weak]
        stack,
        #[strong]
        debounce_source,
        #[strong]
        last_browse,
        move |entry| {
            if let Some(source_id) = debounce_source.borrow_mut().take() {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    source_id.remove();
                }));
            }

            let query = entry.text().to_string().to_lowercase();
            let populate = populate.clone();
            let debounce_source_clone = debounce_source.clone();
            let stack = stack.clone();
            let last_browse = last_browse.clone();

            let source_id =
                glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
                    *debounce_source_clone.borrow_mut() = None;
                    let active = last_browse.borrow().clone();
                    if query.is_empty() {
                        stack.set_visible_child_name(&active);
                    } else if active != "gifs" {
                        // GIF tab keeps its own dedicated search
                        // (see issue #13);
                        // only hijack for the text-based tabs
                        populate(&query);
                        stack.set_visible_child_name(RESULTS_PAGE);
                    }
                });
            *debounce_source.borrow_mut() = Some(source_id);
        }
    ));

    // enter on the results page = insert the top-ranked item
    super::on_search_enter(search_entry, &container, move || {
        if let Some(obj) = selection_model.item(0) {
            if let Ok(item) = obj.downcast::<SearchResultObject>() {
                super::insert_text(item.display(), true);
            }
        }
    });

    container
}
