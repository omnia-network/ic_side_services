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
pub struct GetRawTransaction200ResponseData {
    /// The serialized, hex-encoded data for 'txid'
    #[serde(rename = "hex", skip_serializing_if = "Option::is_none")]
    pub hex: Option<String>,
    /// The transaction id (same as provided)
    #[serde(rename = "txid", skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
    /// The version
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<f32>,
    /// The Overwintered flag
    #[serde(rename = "overwintered", skip_serializing_if = "Option::is_none")]
    pub overwintered: Option<bool>,
    /// The version group id
    #[serde(rename = "versiongroupid", skip_serializing_if = "Option::is_none")]
    pub versiongroupid: Option<String>,
    /// The lock time
    #[serde(rename = "locktime", skip_serializing_if = "Option::is_none")]
    pub locktime: Option<f32>,
    /// Last valid block height for mining transaction
    #[serde(rename = "expiryheight", skip_serializing_if = "Option::is_none")]
    pub expiryheight: Option<f32>,
    #[serde(rename = "vin", skip_serializing_if = "Option::is_none")]
    pub vin: Option<Vec<crate::models::GetRawTransaction200ResponseDataVinInner>>,
    #[serde(rename = "vout", skip_serializing_if = "Option::is_none")]
    pub vout: Option<Vec<crate::models::VoutInner>>,
    #[serde(rename = "vJoinSplit", skip_serializing_if = "Option::is_none")]
    pub v_join_split: Option<Vec<crate::models::JoinsplitInner>>,
    /// The balance value of FLUX
    #[serde(rename = "valueBalance", skip_serializing_if = "Option::is_none")]
    pub value_balance: Option<f32>,
    /// The balance value of FLUX
    #[serde(rename = "valueBalanceZat", skip_serializing_if = "Option::is_none")]
    pub value_balance_zat: Option<i32>,
    #[serde(rename = "vShieldedSpend", skip_serializing_if = "Option::is_none")]
    pub v_shielded_spend: Option<Vec<crate::models::ShieldedspendInner>>,
    #[serde(rename = "vShieldedOutput", skip_serializing_if = "Option::is_none")]
    pub v_shielded_output: Option<Vec<crate::models::ShieldedoutputInner>>,
    /// Binding signature
    #[serde(rename = "bindingSig", skip_serializing_if = "Option::is_none")]
    pub binding_sig: Option<String>,
    /// The block hash
    #[serde(rename = "blockhash", skip_serializing_if = "Option::is_none")]
    pub blockhash: Option<String>,
    /// Block height
    #[serde(rename = "height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// The confirmations
    #[serde(rename = "confirmations", skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<i32>,
    /// The transaction time in seconds since epoch (Jan 1 1970 GMT)
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i32>,
    /// The block time in seconds since epoch (Jan 1 1970 GMT)
    #[serde(rename = "blocktime", skip_serializing_if = "Option::is_none")]
    pub blocktime: Option<i32>,
}

impl GetRawTransaction200ResponseData {
    pub fn new() -> GetRawTransaction200ResponseData {
        GetRawTransaction200ResponseData {
            hex: None,
            txid: None,
            version: None,
            overwintered: None,
            versiongroupid: None,
            locktime: None,
            expiryheight: None,
            vin: None,
            vout: None,
            v_join_split: None,
            value_balance: None,
            value_balance_zat: None,
            v_shielded_spend: None,
            v_shielded_output: None,
            binding_sig: None,
            blockhash: None,
            height: None,
            confirmations: None,
            time: None,
            blocktime: None,
        }
    }
}


