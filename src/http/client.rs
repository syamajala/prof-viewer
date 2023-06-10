use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use reqwest::blocking::{Client, ClientBuilder};
#[cfg(target_arch = "wasm32")]
use reqwest::{Client, ClientBuilder};

use serde::Deserialize;

use url::Url;

use crate::data::{DataSourceInfo, EntryID, SlotMetaTile, SlotTile, SummaryTile, TileID, TileSet};
use crate::deferred_data::DeferredDataSource;
use crate::http::fetch::{fetch, DataSourceResponse};
use crate::http::schema::TileRequestRef;

pub struct HTTPClientDataSource {
    pub baseurl: Url,
    pub client: Client,
    infos: Arc<Mutex<Vec<DataSourceInfo>>>,
    tile_sets: Arc<Mutex<Vec<TileSet>>>,
    summary_tiles: Arc<Mutex<Vec<SummaryTile>>>,
    slot_tiles: Arc<Mutex<Vec<SlotTile>>>,
    slot_meta_tiles: Arc<Mutex<Vec<SlotMetaTile>>>,
}

impl HTTPClientDataSource {
    pub fn new(baseurl: Url) -> Self {
        Self {
            baseurl,
            client: ClientBuilder::new().build().unwrap(),
            infos: Arc::new(Mutex::new(Vec::new())),
            tile_sets: Arc::new(Mutex::new(Vec::new())),
            summary_tiles: Arc::new(Mutex::new(Vec::new())),
            slot_tiles: Arc::new(Mutex::new(Vec::new())),
            slot_meta_tiles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn request<T>(&mut self, url: Url, body: String, container: Arc<Mutex<Vec<T>>>)
    where
        T: 'static + Sync + Send + for<'a> Deserialize<'a>,
    {
        let request = self
            .client
            .post(url)
            .header("Accept", "*/*")
            .header("Content-Type", "javascript/json;")
            .body(body);
        fetch(
            request,
            move |response: Result<DataSourceResponse, String>| {
                let result = serde_json::from_str::<T>(&response.unwrap().body).unwrap();
                container.lock().unwrap().push(result);
            },
        );
    }
}

impl DeferredDataSource for HTTPClientDataSource {
    fn fetch_info(&mut self) {
        let url = self.baseurl.join("/info").expect("invalid baseurl");
        let body = String::new();
        self.request::<DataSourceInfo>(url, body, self.infos.clone());
    }

    fn get_infos(&mut self) -> Vec<DataSourceInfo> {
        std::mem::take(&mut self.infos.lock().unwrap())
    }

    fn fetch_tile_set(&mut self) {
        let url = self.baseurl.join("/tile_set").expect("invalid baseurl");
        let body = String::new();
        self.request::<TileSet>(url, body, self.tile_sets.clone());
    }

    fn get_tile_sets(&mut self) -> Vec<TileSet> {
        std::mem::take(&mut self.tile_sets.lock().unwrap())
    }

    fn fetch_summary_tile(&mut self, entry_id: &EntryID, tile_id: TileID) {
        let url = self.baseurl.join("/summary_tile").expect("invalid baseurl");
        let body = serde_json::to_string(&TileRequestRef { entry_id, tile_id }).unwrap();
        self.request::<SummaryTile>(url, body, self.summary_tiles.clone());
    }

    fn get_summary_tiles(&mut self) -> Vec<SummaryTile> {
        std::mem::take(&mut self.summary_tiles.lock().unwrap())
    }

    fn fetch_slot_tile(&mut self, entry_id: &EntryID, tile_id: TileID) {
        let url = self.baseurl.join("/slot_tile").expect("invalid baseurl");
        let body = serde_json::to_string(&TileRequestRef { entry_id, tile_id }).unwrap();
        self.request::<SlotTile>(url, body, self.slot_tiles.clone());
    }

    fn get_slot_tiles(&mut self) -> Vec<SlotTile> {
        std::mem::take(&mut self.slot_tiles.lock().unwrap())
    }

    fn fetch_slot_meta_tile(&mut self, entry_id: &EntryID, tile_id: TileID) {
        let url = self
            .baseurl
            .join("/slot_meta_tile")
            .expect("invalid baseurl");
        let body = serde_json::to_string(&TileRequestRef { entry_id, tile_id }).unwrap();
        self.request::<SlotMetaTile>(url, body, self.slot_meta_tiles.clone());
    }

    fn get_slot_meta_tiles(&mut self) -> Vec<SlotMetaTile> {
        std::mem::take(&mut self.slot_meta_tiles.lock().unwrap())
    }
}
