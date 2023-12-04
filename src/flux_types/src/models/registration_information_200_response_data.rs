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
pub struct RegistrationInformation200ResponseData {
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<Box<crate::models::RegistrationInformation200ResponseDataPrice>>,
    /// Apps registration address
    #[serde(rename = "address", skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Fluxapps epoch blockheight start
    #[serde(rename = "epochstart", skip_serializing_if = "Option::is_none")]
    pub epochstart: Option<i32>,
    /// Min number in port range
    #[serde(rename = "portMin", skip_serializing_if = "Option::is_none")]
    pub port_min: Option<i32>,
    /// Max number in port range
    #[serde(rename = "portMax", skip_serializing_if = "Option::is_none")]
    pub port_max: Option<i32>,
    /// Max number of image size allowed in bytes
    #[serde(rename = "maxImageSize", skip_serializing_if = "Option::is_none")]
    pub max_image_size: Option<i32>,
    #[serde(rename = "installation", skip_serializing_if = "Option::is_none")]
    pub installation: Option<Box<crate::models::RegistrationInformation200ResponseDataInstallation>>,
    #[serde(rename = "removal", skip_serializing_if = "Option::is_none")]
    pub removal: Option<Box<crate::models::RegistrationInformation200ResponseDataRemoval>>,
    /// Amount of time registered app is live in blocks. 22000 blocks = 1 month
    #[serde(rename = "blocksLasting", skip_serializing_if = "Option::is_none")]
    pub blocks_lasting: Option<i32>,
}

impl RegistrationInformation200ResponseData {
    pub fn new() -> RegistrationInformation200ResponseData {
        RegistrationInformation200ResponseData {
            price: None,
            address: None,
            epochstart: None,
            port_min: None,
            port_max: None,
            max_image_size: None,
            installation: None,
            removal: None,
            blocks_lasting: None,
        }
    }
}


