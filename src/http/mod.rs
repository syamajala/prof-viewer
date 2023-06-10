pub mod schema;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod fetch;
#[cfg(feature = "client")]
#[cfg(not(target_arch = "wasm32"))]
pub mod fetch_native;
#[cfg(feature = "client")]
#[cfg(target_arch = "wasm32")]
pub mod fetch_web;
