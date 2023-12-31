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
pub struct Fluxnodestatus {
    /// FluxNode status
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Collateral transaction
    #[serde(rename = "collateral", skip_serializing_if = "Option::is_none")]
    pub collateral: Option<String>,
    /// Collateral transaction hash
    #[serde(rename = "txhash", skip_serializing_if = "Option::is_none")]
    pub txhash: Option<String>,
    /// Collateral transaction output index number
    #[serde(rename = "outidx", skip_serializing_if = "Option::is_none")]
    pub outidx: Option<i32>,
    /// FluxNode network address
    #[serde(rename = "ip", skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// Network type (IPv4, IPv6, onion)
    #[serde(rename = "network", skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// Block height when FluxNode was added
    #[serde(rename = "added_height", skip_serializing_if = "Option::is_none")]
    pub added_height: Option<i32>,
    /// Block height when FluxNode was confirmed
    #[serde(rename = "confirmed_height", skip_serializing_if = "Option::is_none")]
    pub confirmed_height: Option<i32>,
    /// Last block height when FluxNode was confirmed
    #[serde(rename = "last_confirmed_height", skip_serializing_if = "Option::is_none")]
    pub last_confirmed_height: Option<i32>,
    /// Last block height when FluxNode was paid
    #[serde(rename = "last_paid_height", skip_serializing_if = "Option::is_none")]
    pub last_paid_height: Option<i32>,
    /// Tier (CUMULUS/NIMMBUS/STRATUS)
    #[serde(rename = "tier", skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    /// FLUX address for FluxNode payments
    #[serde(rename = "payment_address", skip_serializing_if = "Option::is_none")]
    pub payment_address: Option<String>,
    /// FluxNode public key used for message broadcasting
    #[serde(rename = "pubkey", skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<String>,
    /// The time in seconds since epoch (Jan 1 1970 GMT) FluxNode has been active
    #[serde(rename = "activesince", skip_serializing_if = "Option::is_none")]
    pub activesince: Option<String>,
    /// The time in seconds since epoch (Jan 1 1970 GMT) FluxNode was last paid
    #[serde(rename = "lastpaid", skip_serializing_if = "Option::is_none")]
    pub lastpaid: Option<String>,
    /// Locked collateral amount
    #[serde(rename = "amount", skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

impl Fluxnodestatus {
    pub fn new() -> Fluxnodestatus {
        Fluxnodestatus {
            status: None,
            collateral: None,
            txhash: None,
            outidx: None,
            ip: None,
            network: None,
            added_height: None,
            confirmed_height: None,
            last_confirmed_height: None,
            last_paid_height: None,
            tier: None,
            payment_address: None,
            pubkey: None,
            activesince: None,
            lastpaid: None,
            amount: None,
        }
    }
}


