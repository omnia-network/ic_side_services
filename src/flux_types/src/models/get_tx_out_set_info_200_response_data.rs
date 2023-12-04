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
pub struct GetTxOutSetInfo200ResponseData {
    /// Current block
    #[serde(rename = "height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Best block hash hex
    #[serde(rename = "bestblock", skip_serializing_if = "Option::is_none")]
    pub bestblock: Option<String>,
    /// The number of transactions
    #[serde(rename = "transactions", skip_serializing_if = "Option::is_none")]
    pub transactions: Option<i32>,
    /// The number of output transactions
    #[serde(rename = "txouts", skip_serializing_if = "Option::is_none")]
    pub txouts: Option<i32>,
    /// The serialized size
    #[serde(rename = "bytes_serialized", skip_serializing_if = "Option::is_none")]
    pub bytes_serialized: Option<i32>,
    /// The serialized hash
    #[serde(rename = "hash_serialized", skip_serializing_if = "Option::is_none")]
    pub hash_serialized: Option<String>,
    /// The total amount
    #[serde(rename = "total_amount", skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<f32>,
}

impl GetTxOutSetInfo200ResponseData {
    pub fn new() -> GetTxOutSetInfo200ResponseData {
        GetTxOutSetInfo200ResponseData {
            height: None,
            bestblock: None,
            transactions: None,
            txouts: None,
            bytes_serialized: None,
            hash_serialized: None,
            total_amount: None,
        }
    }
}


