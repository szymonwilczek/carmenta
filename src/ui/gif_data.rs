use gtk4::glib;
use gtk4::subclass::prelude::*;
use serde::Deserialize;
use std::cell::RefCell;

// to avoid plaintext in repo
const KLIPY_KEY_BYTES: [u8; 64] = [
    83, 67, 68, 110, 88, 102, 103, 90, 66, 98, 108, 76, 78, 79, 117, 117, 
    66, 112, 88, 115, 119, 68, 108, 78, 86, 97, 113, 83, 102, 89, 106, 82, 
    73, 73, 119, 76, 55, 104, 105, 77, 83, 51, 106, 86, 89, 65, 83, 101, 
    112, 108, 77, 120, 78, 68, 52, 77, 48, 100, 78, 77, 57, 81, 51, 118
];

fn get_api_key() -> String {
    String::from_utf8_lossy(&KLIPY_KEY_BYTES).to_string()
}

#[derive(Debug, Deserialize)]
pub struct KlipyApiResponse {
    pub result: bool,
    pub data: KlipyDataWrapper,
}

#[derive(Debug, Deserialize)]
pub struct KlipyDataWrapper {
    pub data: Vec<KlipyGif>,
}

#[derive(Debug, Deserialize)]
pub struct KlipyGif {
    pub id: i64,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub file: KlipyFileFormats,
}

#[derive(Debug, Deserialize)]
pub struct KlipyFileFormats {
    pub hd: Option<KlipyQuality>,
    pub md: Option<KlipyQuality>,
    pub sm: Option<KlipyQuality>,
}

#[derive(Debug, Deserialize)]
pub struct KlipyQuality {
    pub gif: Option<KlipyMedia>,
    pub webp: Option<KlipyMedia>,
}

#[derive(Debug, Deserialize)]
pub struct KlipyMedia {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

// plain data structure (Send + Sync) for threading
#[derive(Debug, Clone)]
pub struct GifData {
    pub id: String,
    pub title: String,
    pub preview_url: String,
    pub full_url: String,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GifObject {
        pub id: RefCell<String>,
        pub title: RefCell<String>,
        pub preview_url: RefCell<String>,
        pub full_url: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GifObject {
        const NAME: &'static str = "CarmentaGifObject";
        type Type = super::GifObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for GifObject {}
}

glib::wrapper! {
    pub struct GifObject(ObjectSubclass<imp::GifObject>);
}

impl GifObject {
    pub fn new(id: String, title: String, preview_url: String, full_url: String) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().id.borrow_mut() = id;
        *obj.imp().title.borrow_mut() = title;
        *obj.imp().preview_url.borrow_mut() = preview_url;
        *obj.imp().full_url.borrow_mut() = full_url;
        obj
    }

    pub fn from_data(data: GifData) -> Self {
        Self::new(data.id, data.title, data.preview_url, data.full_url)
    }

    pub fn id(&self) -> String {
        self.imp().id.borrow().clone()
    }

    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
    }

    pub fn preview_url(&self) -> String {
        self.imp().preview_url.borrow().clone()
    }

    pub fn full_url(&self) -> String {
        self.imp().full_url.borrow().clone()
    }
}

// helper to extract URLs from Klipy file formats
fn extract_gif_data(gif: KlipyGif) -> Option<GifData> {
    // prefer small for faster loading
    let preview_url = gif.file.sm
        .as_ref()
        .and_then(|q| q.gif.as_ref())
        .or_else(|| gif.file.md.as_ref().and_then(|q| q.gif.as_ref()))
        .or_else(|| gif.file.hd.as_ref().and_then(|q| q.gif.as_ref()))?
        .url
        .clone();

    // prefer HD for copying
    let full_url = gif.file.hd
        .as_ref()
        .and_then(|q| q.gif.as_ref())
        .or_else(|| gif.file.md.as_ref().and_then(|q| q.gif.as_ref()))?
        .url
        .clone();

    Some(GifData {
        id: gif.id.to_string(),
        title: gif.title.unwrap_or_else(|| gif.slug.unwrap_or_default()),
        preview_url,
        full_url,
    })
}

pub async fn search_gifs(query: &str) -> Result<Vec<GifData>, reqwest::Error> {
    let url = format!(
        "https://api.klipy.com/api/v1/{}/gifs/search?q={}&limit=30",
        get_api_key(),
        urlencoding::encode(query)
    );

    let client = reqwest::Client::new();
    let response: KlipyApiResponse = client.get(&url).send().await?.json().await?;

    let gifs = response.data.data
        .into_iter()
        .filter_map(extract_gif_data)
        .collect();

    Ok(gifs)
}

pub async fn get_trending_gifs() -> Result<Vec<GifData>, reqwest::Error> {
    let url = format!(
        "https://api.klipy.com/api/v1/{}/gifs/trending?limit=30",
        get_api_key()
    );

    let client = reqwest::Client::new();
    let response: KlipyApiResponse = client.get(&url).send().await?.json().await?;

    let gifs = response.data.data
        .into_iter()
        .filter_map(extract_gif_data)
        .collect();

    Ok(gifs)
}
