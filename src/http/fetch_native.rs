use reqwest::blocking::RequestBuilder;

use crate::http::fetch::DataSourceResponse;

pub fn fetch(
    request: RequestBuilder,
    on_done: Box<dyn FnOnce(Result<DataSourceResponse, String>) + Send>,
) {
    rayon::spawn(move || {
        let result = request
            .send()
            .expect("request failed")
            .bytes()
            .expect("unable to get bytes");

        on_done(Ok(DataSourceResponse { body: result }))
    });
}
