use ic_cdk::{init, post_upgrade};

use ws::init_ws;

mod http_over_ws;
mod ws;

#[init]
fn init() {
    init_ws();
}

#[post_upgrade]
fn post_upgrade() {
    init_ws();
}
