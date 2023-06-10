use serde::{Deserialize, Serialize};

use crate::data::{EntryID, TileID};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRequest {
    pub entry_id: EntryID,
    pub tile_id: TileID,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "TileRequest")]
pub struct TileRequestRef<'a> {
    pub entry_id: &'a EntryID,
    pub tile_id: TileID,
}
