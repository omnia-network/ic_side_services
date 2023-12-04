/*
 * Flux
 *
 * This is an API documentation of calls available to be made to any Flux. <br> GET Calls are available as both query and in order as path. <br> Flux is completely open source and we encourage everyone to feel free and contribute :) <br> Further questions or support join and ask in our [discord](https://discord.io/runonflux)  # Introduction Flux possesses a 5 tier hiearchy level API. * **Public** API level - Available without any permission, does not require signing. * **User** API level - User level permission, requires signing. * **FluxTeam** API level - FluxTeam level permission (an appointed Flux Team member has access to those API calls), requires signing. * **Admin** API level - Admin level permission, requires signing. Flux owner. * **AdminAndFluxTeam** API level permission (Admin and Flux Team has access to these calls), requires signing. * **AppOwner** API level - AppOwner level permission, requires signing. App Owner. * **AppOwnerAbove** API level - AppOwnerAbove level permission (App Owner, FluxTeam, and Admin has access to these calls), requires signing.  Most calls are available via GET request with some that may require large amount of data via POST request. Websocket is currently used only for simplifying login operations and for internal Flux communication. # Getting Started with the API * **1. Install Zelcore Wallet:** The Zelcore wallet is required for signing messages. Please install this if not already installed. * **2. Create Zelcore Account:** A Zelcore wallet account is required for signing messages. Please register an account if you don't already have one. * **3. Obtain API Authentication Credentials:** Follow the Authentication section of this API documentation in order to set up your credentials for using the rest of the API. You will need to use your Zelcore wallet for parts of this process. * **4. Set Up Authentication Credentials in Header:** Set up your zelidauth credentials in your header. If you are using variables, the signature value may need to be URLencoded.
 *
 * The version of the OpenAPI document: 4.9.1
 * Contact: tadeas@runonflux.io
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetFilteredFluxNodeTxs200ResponseData {
    /// The transaction id
    #[serde(rename = "txid", skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
    /// Version number
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    /// Type of transaction
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Update type number
    #[serde(rename = "updateType", skip_serializing_if = "Option::is_none")]
    pub update_type: Option<i32>,
    /// Ip of fluxnode's server
    #[serde(rename = "ip", skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// Fluxnode tier
    #[serde(rename = "benchTier", skip_serializing_if = "Option::is_none")]
    pub bench_tier: Option<String>,
    /// Collateral tx hash
    #[serde(rename = "collateralHash", skip_serializing_if = "Option::is_none")]
    pub collateral_hash: Option<String>,
    /// Collateral tx output index
    #[serde(rename = "collateralIndex", skip_serializing_if = "Option::is_none")]
    pub collateral_index: Option<i32>,
    /// Flux address
    #[serde(rename = "fluxAddress", skip_serializing_if = "Option::is_none")]
    pub flux_address: Option<String>,
    /// Locked amount in satoshi value
    #[serde(rename = "lockedAmount", skip_serializing_if = "Option::is_none")]
    pub locked_amount: Option<i32>,
    /// Block height tx was included in
    #[serde(rename = "height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

impl GetFilteredFluxNodeTxs200ResponseData {
    pub fn new() -> GetFilteredFluxNodeTxs200ResponseData {
        GetFilteredFluxNodeTxs200ResponseData {
            txid: None,
            version: None,
            r#type: None,
            update_type: None,
            ip: None,
            bench_tier: None,
            collateral_hash: None,
            collateral_index: None,
            flux_address: None,
            locked_amount: None,
            height: None,
        }
    }
}


