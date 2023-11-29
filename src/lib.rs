#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod archive_data;
pub mod data;
pub mod deferred_data;
pub mod sql_dump;
#[cfg(not(target_arch = "wasm32"))]
pub mod file_data;
pub mod http;
pub mod merge_data;
#[cfg(not(target_arch = "wasm32"))]
pub mod parallel_data;
pub mod timestamp;
