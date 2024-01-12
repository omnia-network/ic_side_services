use ic_cdk::{
    api::management_canister::main::{canister_info, CanisterInfoRequest},
    caller, id, trap,
};

pub async fn caller_is_controller() -> bool {
    let caller = caller();
    canister_info(CanisterInfoRequest {
        canister_id: id(),
        num_requested_changes: None,
    })
    .await
    .is_ok_and(|info| info.0.controllers.contains(&caller))
}

pub async fn guard_caller_is_controller() {
    if !caller_is_controller().await {
        trap("Caller is not a controller");
    }
}
