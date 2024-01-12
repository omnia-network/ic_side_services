use std::cell::RefCell;

mod state;
mod handlers;
mod http_connection;

use state::State;

// re-exports
pub use handlers::*;
pub use http_connection::*;

// local state
thread_local! {
    /* flexible */ static STATE: RefCell<State> = RefCell::new(State::new());
}
