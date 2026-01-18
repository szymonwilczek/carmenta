use gtk4::prelude::*;
use gtk4::{
    gio, glib, GridView, SignalListItemFactory, SingleSelection,
    PolicyType, ScrolledWindow, Box, Orientation, Spinner
};
use super::gif_data::{GifObject, GifData, search_gifs, get_trending_gifs};
use crate::dbus::DBusClient;
use std::cell::RefCell;
use std::rc::Rc;

// helper function: copy URL and insert via extension
fn insert_gif_url(url: String) {
    crate::app::mark_inserting();
    DBusClient::insert_or_copy(&url);
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
        .visible(false)  // shown only during loading
        .build();
    
    container.append(&spinner);

    let store = gio::ListStore::new::<GifObject>();
    let selection_model = SingleSelection::new(Some(store.clone()));
    let factory = SignalListItemFactory::new();
    
    factory.connect_setup(move |_factory, item| {
        let button = gtk4::Button::builder()
            .css_classes(["gif-btn", "flat"])
            .build();

        let picture = gtk4::Picture::builder()
            .width_request(100)
            .height_request(100)
            .build();
        
        button.set_child(Some(&picture));
        item.set_child(Some(&button));

        // click handler - copy URL
        button.connect_clicked(move |btn| {
            // url from widget name
            let url = btn.widget_name();
            if !url.is_empty() {
                insert_gif_url(url.to_string());
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
                let client = reqwest::Client::new();
                match client.get(&preview_url).send().await {
                    Ok(response) => response.bytes().await.ok().map(|b| (b, gif_id)),
                    Err(e) => {
                        eprintln!("Failed to fetch GIF: {}", e);
                        None
                    }
                }
            },
            move |result_opt| {
                if let Some((bytes, id)) = result_opt {
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

                        let temp_dir = std::env::temp_dir();
                        let temp_path = temp_dir.join(format!("carmenta_gif_{}.gif", id));
                        
                        if std::fs::write(&temp_path, &bytes).is_ok() {
                            let file = gio::File::for_path(&temp_path);
                            let media = gtk4::MediaFile::for_file(&file);
                            media.set_loop(true);
                            media.play();
                            pic.set_paintable(Some(&media));
                        }
                    }
                }
            }
        );
    });

    // cleanup MediaFile when item is unbound
    // without it there is loud and sexy segfault
    factory.connect_unbind(move |_factory, item| {
        if let Some(button) = item.child() {
            if let Ok(button) = button.downcast::<gtk4::Button>() {
                button.set_widget_name("");
                
                if let Some(picture) = button.child() {
                    if let Ok(picture) = picture.downcast::<gtk4::Picture>() {
                        // get current paintable & stop it
                        if let Some(paintable) = picture.paintable() {
                            if let Ok(media) = paintable.downcast::<gtk4::MediaFile>() {
                                media.set_playing(false); // stop playing immediately
                            }
                        }
                        // clear the paintable
                        picture.set_paintable(None::<&gtk4::gdk::Paintable>);
                    }
                }
            }
        }
    });

    let grid_view = GridView::builder()
        .model(&selection_model)
        .factory(&factory)
        .max_columns(4)
        .min_columns(3)
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
    let store_weak = store.downgrade();
    let spinner_weak = spinner.downgrade();
    
    search_entry.connect_search_changed(glib::clone!(
        #[strong] debounce_source,
        #[strong] store_weak,
        #[strong] spinner_weak,
        move |entry| {
            // cancel previous debounce timer
            if let Some(source_id) = debounce_source.borrow_mut().take() {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    source_id.remove();
                }));
            }

            let query = entry.text().to_string();
            let store_weak_clone = store_weak.clone();
            let spinner_weak_clone = spinner_weak.clone();
            let debounce_source_clone = debounce_source.clone();

            // start debounce timer
            let source_id = glib::timeout_add_local_once(
                std::time::Duration::from_millis(300),
                move || {
                    *debounce_source_clone.borrow_mut() = None;
                    
                    if let Some(spinner) = spinner_weak_clone.upgrade() {
                        spinner.set_visible(true);
                        spinner.set_spinning(true);
                    }
                    
                    let store_weak_final = store_weak_clone.clone();
                    let spinner_weak_final = spinner_weak_clone.clone();
                    let query_clone = query.clone();
                    
                    spawn_tokio(
                        async move {
                            if query_clone.is_empty() {
                                get_trending_gifs().await
                            } else {
                                search_gifs(&query_clone).await
                            }
                        },
                        move |results| {
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
                                    }
                                    Err(e) => {
                                        eprintln!("GIF search error: {}", e);
                                    }
                                }
                            }
                        }
                    );
                }
            );
            *debounce_source.borrow_mut() = Some(source_id);
        }
    ));

    // load trending GIFs on startup
    let store_init = store.clone();
    let spinner_init = spinner.clone();
    spinner_init.set_visible(true);
    spinner_init.set_spinning(true);
    
    spawn_tokio(
        async move {
            get_trending_gifs().await
        },
        move |results| {
            spinner_init.set_spinning(false);
            spinner_init.set_visible(false);
            match results {
                Ok(gif_data_list) => {
                    for gif_data in gif_data_list {
                        store_init.append(&GifObject::from_data(gif_data));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load trending GIFs: {}", e);
                }
            }
        }
    );

    container
}

