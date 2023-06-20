use reqwest::RequestBuilder;

use crate::http::fetch::DataSourceResponse;

/// Spawn an async task.
///
/// A wrapper around `wasm_bindgen_futures::spawn_local`.
/// Only available with the web backend.
pub fn spawn_future<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

pub fn fetch(
    request: RequestBuilder,
    on_done: Box<dyn FnOnce(Result<DataSourceResponse, String>) + Send>,
) {
    spawn_future(async move {
        let result = request
            .send()
            .await
            .expect("request failed")
            .bytes()
            .await
            .expect("unable to get bytes");

        let res = Ok(DataSourceResponse { body: result });

        on_done(res)
    });
}
