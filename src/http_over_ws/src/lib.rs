use std::{cell::RefCell, collections::BTreeMap};

mod clients;
mod handlers;
mod types;

use clients::ConnectedClients;

// re-exports
pub use handlers::*;
pub use types::*;

// local state
thread_local! {
    /* flexible */ static HTTP_REQUESTS: RefCell<BTreeMap<HttpRequestId, HttpRequestState>> = RefCell::new(BTreeMap::new());
    /* flexible */ static CONNECTED_CLIENTS: RefCell<ConnectedClients> = RefCell::new(ConnectedClients::new());
}
