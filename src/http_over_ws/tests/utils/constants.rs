use http_over_ws::HttpHeader;
use lazy_static::lazy_static;

pub const TEST_URL: &str = "https://example.com/";

lazy_static! {
    pub static ref TEST_HEADER: HttpHeader = HttpHeader {
        name: String::from("Content-Type"),
        value: String::from("text/plain"),
    };
}
