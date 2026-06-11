use super::gif_data::{get_trending_gifs, search_gifs, GifObject};
use crate::dbus::DBusClient;
use gtk4::prelude::*;
use gtk4::{
    gio, glib, Box, GridView, Orientation, PolicyType, ScrolledWindow, SignalListItemFactory,
    SingleSelection, Spinner,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

// helper function: copy URL and insert via extension
fn insert_gif_url(url: String, close_after: bool) {
    crate::app::mark_inserting();
    DBusClient::insert_or_copy(&url, close_after);
}

// helper to run async code on tokio runtime and return result to GTK main loop
fn spawn_tokio<F, T>(future: F, callback: impl FnOnce(T) + 'static)
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();

    if let Some(rt) = crate::RUNTIME.get() {
        rt.spawn(async move {
            let result = future.await;
            let _ = tx.send(result);
        });
    }

    // cell to allow moving FnOnce out of the closure
    let callback = std::cell::Cell::new(Some(callback));

    // poll for result on GTK main loop
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(result) => {
                if let Some(cb) = callback.take() {
                    cb(result);
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
        }
    });
}

pub fn create_gif_grid(search_entry: &gtk4::SearchEntry) -> Box {
    let container = Box::new(Orientation::Vertical, 0);
    container.set_css_classes(&["gif-page"]);

    // shown during API requests
    let spinner = Spinner::builder()
        .spinning(false)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .width_request(32)
        .height_request(32)
        .visible(false) // shown only during loading
        .build();

    container.append(&spinner);

    let store = gio::ListStore::new::<GifObject>();
    let selection_model = SingleSelection::new(Some(store.clone()));
    let factory = SignalListItemFactory::new();

    factory.connect_setup(move |_factory, item| {
        let button = gtk4::Button::builder()
            .css_classes(["gif-btn", "flat"])
            .build();

        // GIFs are images, not text, so they cant ride the CSS font-size
        // scaling
        // Scale the requested thumbnail size directly instead
        let gif_size = (100.0 * crate::scale()).round() as i32;
        let picture = gtk4::Picture::builder()
            .width_request(gif_size)
            .height_request(gif_size)
            .build();

        button.set_child(Some(&picture));
        item.set_child(Some(&button));

        // click handler - copy URL
        button.connect_clicked(move |btn| {
            // url from widget name
            let url = btn.widget_name();
            if !url.is_empty() {
                insert_gif_url(url.to_string(), crate::close_on_select());
            }
        });
    });

    factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let button = item.child().unwrap().downcast::<gtk4::Button>().unwrap();
        let picture = button.child().unwrap().downcast::<gtk4::Picture>().unwrap();
        let gif_obj = item.item().unwrap().downcast::<GifObject>().unwrap();

        let preview_url = gif_obj.preview_url();
        let full_url = gif_obj.full_url();
        let gif_id = gif_obj.id();

        // store full URL in widget name
        button.set_widget_name(&full_url);

        // load GIF asynchronously
        let picture_weak = picture.downgrade();
        let full_url_check = full_url.clone();

        spawn_tokio(
            async move {
                match crate::client().get(&preview_url).send().await {
                    Ok(response) => response.bytes().await.ok().map(|b| (b, gif_id)),
                    Err(e) => {
                        eprintln!("Failed to fetch GIF: {}", e);
                        None
                    }
                }
            },
            move |result_opt| {
                if let Some((bytes, _id)) = result_opt {
                    if let Some(pic) = picture_weak.upgrade() {
                        if let Some(parent) = pic.parent() {
                            if let Ok(btn) = parent.downcast::<gtk4::Button>() {
                                if btn.widget_name() != full_url_check {
                                    // widget reused for another item, discard result
                                    return;
                                }

                                // check if widget is still in the component tree
                                if pic.root().is_none() {
                                    return;
                                }
                            } else {
                                return;
                            }
                        } else {
                            return;
                        }

                        // Use PixbufAnimation to drive the animation manually.
                        // This avoids GStreamer pipelines entirely while keeping the animation.
                        let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&bytes));
                        if let Ok(anim) = gdk_pixbuf::PixbufAnimation::from_stream(
                            &stream,
                            None::<&gio::Cancellable>,
                        ) {
                            let iter = anim.iter(None);

                            // drive the first frame
                            let pixbuf = iter.pixbuf();
                            let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
                            pic.set_paintable(Some(&texture));

                            // advance animation on GTK main loop with per-frame delay
                            let initial_delay =
                                iter.delay_time().map(|d| d.as_millis()).unwrap_or(100);
                            let pic_weak_loop = pic.downgrade();
                            glib::timeout_add_local(
                                std::time::Duration::from_millis(initial_delay as u64),
                                move || {
                                    if let Some(p) = pic_weak_loop.upgrade() {
                                        if p.paintable().is_some() {
                                            iter.advance(SystemTime::now());
                                            let pixbuf = iter.pixbuf();
                                            let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
                                            p.set_paintable(Some(&texture));
                                            return glib::ControlFlow::Continue;
                                        }
                                    }
                                    glib::ControlFlow::Break
                                },
                            );
                        }
                    }
                }
            },
        );
    });

    // cleanup paintable when item is unbound
    // clearing paintable terminates the glib::timeout animation loop
    factory.connect_unbind(move |_factory, item| {
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        if let Some(button) = item.child() {
            if let Ok(button) = button.downcast::<gtk4::Button>() {
                button.set_widget_name("");
                if let Some(picture) = button.child() {
                    if let Ok(picture) = picture.downcast::<gtk4::Picture>() {
                        // Clearing the paintable stops the glib::timeout loop
                        picture.set_paintable(None::<&gtk4::gdk::Paintable>);
                    }
                }
            }
        }
    });

    // Larger thumbnails need fewer columns so the row still fits the window
    // width (the scrolled window never shows a horizontal scrollbar)
    let scale = crate::scale();
    let (min_cols, max_cols) = if scale <= 1.0 {
        (3, 4)
    } else if scale <= 1.5 {
        (2, 3)
    } else {
        (1, 2)
    };
    let grid_view = GridView::builder()
        .model(&selection_model)
        .factory(&factory)
        .max_columns(max_cols)
        .min_columns(min_cols)
        .build();

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&grid_view)
        .hexpand(true)
        .vexpand(true)
        .build();

    container.append(&scrolled_window);

    // search with debounce (300ms)
    let debounce_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let search_pending = Rc::new(RefCell::new(false));
    let loaded_query: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let store_weak = store.downgrade();
    let spinner_weak = spinner.downgrade();

    search_entry.connect_search_changed(glib::clone!(
        #[strong]
        debounce_source,
        #[strong]
        search_pending,
        #[strong]
        loaded_query,
        #[strong]
        store_weak,
        #[strong]
        spinner_weak,
        move |entry| {
            // cancel previous debounce timer
            if let Some(source_id) = debounce_source.borrow_mut().take() {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    source_id.remove();
                }));
            }

            let query = entry.text().to_string();
            *search_pending.borrow_mut() = true;
            let store_weak_clone = store_weak.clone();
            let spinner_weak_clone = spinner_weak.clone();
            let debounce_source_clone = debounce_source.clone();
            let search_pending_clone = search_pending.clone();
            let loaded_query_clone = loaded_query.clone();

            // start debounce timer
            let source_id =
                glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                    *debounce_source_clone.borrow_mut() = None;

                    if let Some(spinner) = spinner_weak_clone.upgrade() {
                        spinner.set_visible(true);
                        spinner.set_spinning(true);
                    }

                    let store_weak_final = store_weak_clone.clone();
                    let spinner_weak_final = spinner_weak_clone.clone();
                    let query_for_request = query.clone();
                    let query_for_result = query.clone();

                    spawn_tokio(
                        async move {
                            if query_for_request.is_empty() {
                                get_trending_gifs().await
                            } else {
                                search_gifs(&query_for_request).await
                            }
                        },
                        move |results| {
                            *search_pending_clone.borrow_mut() = false;
                            if let Some(spinner) = spinner_weak_final.upgrade() {
                                spinner.set_spinning(false);
                                spinner.set_visible(false);
                            }

                            if let Some(store) = store_weak_final.upgrade() {
                                match results {
                                    Ok(gif_data_list) => {
                                        store.remove_all();
                                        for gif_data in gif_data_list {
                                            store.append(&GifObject::from_data(gif_data));
                                        }
                                        *loaded_query_clone.borrow_mut() = Some(query_for_result);
                                    }
                                    Err(e) => {
                                        eprintln!("GIF search error: {}", e);
                                    }
                                }
                            }
                        },
                    );
                });
            *debounce_source.borrow_mut() = Some(source_id);
        }
    ));

    // Load trending GIFs lazily, the first time the GIF page is actually shown.
    // This keeps the HTTP client (TLS init) and the network request off the
    // startup path entirely when the user only wants an emoji/symbol.
    let loaded = Rc::new(RefCell::new(false));
    container.connect_map(glib::clone!(
        #[strong]
        store,
        #[strong]
        search_entry,
        #[weak]
        spinner,
        #[strong]
        loaded,
        #[strong]
        search_pending,
        #[strong]
        loaded_query,
        move |_| {
            if *loaded.borrow() {
                return;
            }
            if !search_entry.text().is_empty() {
                return;
            }
            *loaded.borrow_mut() = true;

            *search_pending.borrow_mut() = true;
            spinner.set_visible(true);
            spinner.set_spinning(true);
            let store_init = store.clone();
            let search_pending_done = search_pending.clone();
            let loaded_query_done = loaded_query.clone();
            spawn_tokio(async move { get_trending_gifs().await }, move |results| {
                *search_pending_done.borrow_mut() = false;
                spinner.set_spinning(false);
                spinner.set_visible(false);
                match results {
                    Ok(gif_data_list) => {
                        for gif_data in gif_data_list {
                            store_init.append(&GifObject::from_data(gif_data));
                        }
                        *loaded_query_done.borrow_mut() = Some(String::new());
                    }
                    Err(e) => {
                        eprintln!("Failed to load trending GIFs: {}", e);
                    }
                }
            });
        }
    ));

    // Enter in the search box = select the first loaded GIF and close.
    {
        let selection_model = selection_model.clone();
        let entry = search_entry.clone();
        let search_pending = search_pending.clone();
        let loaded_query = loaded_query.clone();
        super::on_search_enter(search_entry, &container, move || {
            let query = entry.text().to_string();
            if *search_pending.borrow() || loaded_query.borrow().as_deref() != Some(query.as_str())
            {
                return;
            }
            if let Some(obj) = selection_model.item(0) {
                if let Ok(gif) = obj.downcast::<GifObject>() {
                    let url = gif.full_url();
                    if !url.is_empty() {
                        insert_gif_url(url, true);
                    }
                }
            }
        });
    }

    container
}
