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
pub struct AppregisterRequest {
    /// Message type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Message version
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    #[serde(rename = "appSpecification", skip_serializing_if = "Option::is_none")]
    pub app_specification: Option<Box<crate::models::Appspecification>>,
    /// Current timestamp in epoch/unix
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    /// Signature of data message you signed with ZelID.
    #[serde(rename = "signature", skip_serializing_if = "Option::is_none")]
    pub signature: Option<serde_json::Value>,
}

impl AppregisterRequest {
    pub fn new() -> AppregisterRequest {
        AppregisterRequest {
            r#type: None,
            version: None,
            app_specification: None,
            timestamp: None,
            signature: None,
        }
    }
}


