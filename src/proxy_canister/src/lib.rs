mod constants;
mod requests;
mod state;
mod utils;
mod ws;

use http_over_ws::{execute_http_request, HttpRequestId, HttpResponse};
use ic_cdk::caller;
use ic_cdk_macros::*;
use logger::log;
use proxy_canister_types::{
    CanisterRequest, HttpRequestEndpointArgs, HttpRequestEndpointResult, ProxyError, RequestState,
};
use requests::validate_incoming_request;
use std::{cell::RefCell, future::Future, pin::Pin};

use crate::{
    state::ProxyState,
    utils::{guard_caller_is_controller, guard_caller_is_not_anonymous},
};

thread_local! {
    /* flexible */ static STATE: RefCell<ProxyState> = RefCell::new(ProxyState::new());
}

#[init]
fn init() {
    ws::init_ws();
}

#[post_upgrade]
fn post_upgrade() {
    init();
}

#[update]
fn http_request(args: HttpRequestEndpointArgs) -> HttpRequestEndpointResult {
    let canister_id = caller();

    guard_caller_is_not_anonymous(&canister_id);

    validate_incoming_request(&args).map_err(|e| ProxyError::InvalidRequest(e))?;

    log!(
        "[http_request]: canister_id:{}, incoming request valid",
        canister_id
    );

    let request_id = execute_http_request(
        args.request,
        // using .map() doesn't work because it becomes a closure
        match args.callback_method_name {
            Some(_) => Some(cb),
            None => None,
        },
        args.timeout_ms,
        ws::send,
    )
    .map_err(|_| ProxyError::Generic(String::from("http_over_ws error")))?;

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        state.start_request_for_canister(
            canister_id,
            request_id,
            args.callback_method_name.clone(),
        );
    });

    log!(
        "[http_request]: request_id:{}, canister_id:{}, timeout_ms:{:?}, callback method:{:?}, started",
        request_id,
        canister_id,
        args.timeout_ms,
        args.callback_method_name
    );

    Ok(request_id)
}

fn cb(id: HttpRequestId, res: HttpResponse) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(call_canister_endpoint_callback(id, res))
}

async fn call_canister_endpoint_callback(request_id: HttpRequestId, res: HttpResponse) {
    let request_state = STATE.with(|state| state.borrow().get_request_state(request_id));

    if let Some(r) = request_state {
        log!(
            "[http_request]: request_id:{}, canister_id:{}, http completed",
            request_id,
            r.canister_id
        );

        if let RequestState::Executing(Some(method_name)) = r.state {
            log!(
                "[http_request]: request_id:{}, canister_id:{}, callback method:{}, starting inter-canister call",
                request_id,
                r.canister_id,
                method_name
            );

            let canister_res: Result<(), _> =
                ic_cdk::call(r.canister_id, method_name.as_str(), (request_id, res)).await;

            log!(
                "[http_request]: request_id:{}, canister_id:{}, callback method:{}, completed inter-canister call result: {:?}",
                request_id,
                r.canister_id,
                method_name,
                canister_res
            );

            STATE.with(|state| {
                let mut state = state.borrow_mut();

                match canister_res {
                    Ok(_) => {
                        state.set_request_successful(request_id);
                    }
                    Err(e) => {
                        state.set_request_failed(request_id, format!("{:?}", e));
                    }
                };
            });

            log!(
                "[http_request]: request_id:{}, canister_id:{}, completed",
                request_id,
                r.canister_id
            );
        }
    } else {
        log!("[http_request]: request {} not found", request_id);
    }
}

#[query]
async fn get_request_by_id(request_id: HttpRequestId) -> Option<CanisterRequest> {
    let caller = caller();
    guard_caller_is_controller(&caller).await;

    STATE.with(|state| state.borrow().get_request_state(request_id))
}

#[query]
async fn get_logs() -> Vec<(String, String)> {
    let caller = caller();
    guard_caller_is_controller(&caller).await;

    logger::get_logs()
}