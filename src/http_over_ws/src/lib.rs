use std::cell::RefCell;

mod clients;
mod handlers;
mod types;

use clients::State;

// re-exports
pub use handlers::*;
pub use types::*;

// local state
thread_local! {
    /* flexible */ static STATE: RefCell<State> = RefCell::new(State::new());
}
