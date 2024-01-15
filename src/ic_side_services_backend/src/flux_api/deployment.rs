use crate::{
    flux,
    flux_api::{
        authentication::get_zelidauth_or_trap, CONTENT_TYPE_TEXT_PLAIN_HEADER,
        DEFAULT_HTTP_REQUEST_TIMEOUT_MS, FLUX_API_BASE_URL,
    },
    http_over_ws::execute_http_request,
    logger::log,
    sign_with_ecdsa, utils, NETWORK,
};
use flux_types::models::*;
use proxy_canister_types::{HttpMethod, HttpRequestEndpointResult};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

pub type ComposeSpec = GetAppPriceRequestComposeInner;

pub struct DeploymentInfo {
    /// Similar to a docker-compose specification
    pub compose: ComposeSpec,
    /// On how many nodes the app should be deployed
    pub instances: i32,
    /// How many blocks the app should be valid for
    ///
    /// 5000 blocks are ~ 1 week (according to Flux API)
    pub expires_after_blocks: i32,
    /// If the app should have a static ip
    pub static_ip: bool,
}

/// See https://docs.runonflux.io/#tag/Apps/operation/getAppPrice.
pub async fn calculate_app_price(deployment_info: DeploymentInfo) -> HttpRequestEndpointResult {
    let calculateprice_url = FLUX_API_BASE_URL.join("/apps/calculateprice").unwrap();

    let body = GetAppPriceRequest {
        version: Some(7),
        name: deployment_info.compose.clone().name,
        description: deployment_info.compose.clone().description,
        owner: Some(flux::get_p2pkh_address(
            NETWORK.with(|n| n.get()),
            flux::P2PKHAddress::ZelId,
        )),
        compose: Some(vec![deployment_info.compose]),
        instances: Some(deployment_info.instances),
        contacts: Some(vec![]),
        geolocation: Some(vec![]),
        expire: Some(deployment_info.expires_after_blocks),
        nodes: Some(vec![]),
        staticip: Some(deployment_info.static_ip),
    };

    execute_http_request(
        calculateprice_url,
        HttpMethod::POST,
        vec![CONTENT_TYPE_TEXT_PLAIN_HEADER.deref().clone()],
        Some(serde_json::to_vec(&body).unwrap()),
        Some(String::from("calculate_price_callback")),
        Some(DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    )
    .await
}

/// See https://docs.runonflux.io/#tag/Apps/operation/Appregister.
pub async fn register_app(deployment_info: DeploymentInfo) -> HttpRequestEndpointResult {
    let zelidauth = get_zelidauth_or_trap();
    let appregister_url = FLUX_API_BASE_URL.join("/apps/appregister").unwrap();

    let mut body = AppregisterRequest {
        r#type: Some("fluxappregister".to_string()),
        version: Some(1),
        app_specification: Some(Box::new(Appspecification {
            version: Some(7),
            name: deployment_info.compose.clone().name,
            description: deployment_info.compose.clone().description,
            owner: Some(flux::get_p2pkh_address(
                NETWORK.with(|n| n.get()),
                flux::P2PKHAddress::ZelId,
            )),
            compose: Some(vec![deployment_info.compose]),
            instances: Some(deployment_info.instances),
            contacts: Some(vec![]),
            geolocation: Some(vec![]),
            expire: Some(deployment_info.expires_after_blocks), // 5000 blocks are ~ 1 week (according to Flux API)
            nodes: Some(vec![]),
            staticip: Some(deployment_info.static_ip),
        })),
        timestamp: Some(i64::try_from(utils::get_current_timestamp_ms()).unwrap()),
        signature: None,
    };

    let to_sign = [
        body.r#type.clone().unwrap(),
        body.version.unwrap().to_string(),
        serde_json::to_string(&body.app_specification.clone().unwrap()).unwrap(),
        body.timestamp.unwrap().to_string(),
    ]
    .join("");

    log(&format!("appregister to sign: {}", to_sign));

    let signature = sign_with_ecdsa(to_sign, None).await;

    body.signature = Some(serde_json::Value::String(signature));

    execute_http_request(
        appregister_url,
        HttpMethod::POST,
        vec![CONTENT_TYPE_TEXT_PLAIN_HEADER.deref().clone(), zelidauth],
        Some(serde_json::to_vec(&body).unwrap()),
        Some(String::from("app_register_callback")),
        // this request can take longer to complete due to the sign_with_ecdsa in the callback
        Some(2 * DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    )
    .await
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeploymentInformationData {
    pub address: String,
}

/// For some reason, this is not generated from the OpenAPI spec.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeploymentInformationResponse {
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /// Status of the cleanup in progress
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<DeploymentInformationData>,
}

/// See https://docs.runonflux.io/#tag/Apps/operation/getDeploymentInformatio.
pub async fn fetch_deployment_information() -> HttpRequestEndpointResult {
    let deploymentinformation_url = FLUX_API_BASE_URL
        .join("/apps/deploymentinformation")
        .unwrap();

    execute_http_request(
        deploymentinformation_url,
        HttpMethod::GET,
        vec![CONTENT_TYPE_TEXT_PLAIN_HEADER.deref().clone()],
        None,
        Some(String::from("deployment_information_callback")),
        Some(DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    )
    .await
}
