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
pub struct MountsInner {
    /// Mount type
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    /// Path of dir to mount
    #[serde(rename = "Source", skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Dir of container to bind mount
    #[serde(rename = "Destination", skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    /// Bind mount mode
    #[serde(rename = "Mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Whether the mount should be read-write
    #[serde(rename = "RW", skip_serializing_if = "Option::is_none")]
    pub rw: Option<bool>,
    /// A propagation mode with the value [r]private, [r]shared, or [r]slave.
    #[serde(rename = "Propagation", skip_serializing_if = "Option::is_none")]
    pub propagation: Option<Propagation>,
}

impl MountsInner {
    pub fn new() -> MountsInner {
        MountsInner {
            r#type: None,
            source: None,
            destination: None,
            mode: None,
            rw: None,
            propagation: None,
        }
    }
}

/// Mount type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "bind")]
    Bind,
    #[serde(rename = "volume")]
    Volume,
    #[serde(rename = "tmpfs")]
    Tmpfs,
    #[serde(rename = "npipe")]
    Npipe,
}

impl Default for Type {
    fn default() -> Type {
        Self::Bind
    }
}
/// A propagation mode with the value [r]private, [r]shared, or [r]slave.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Propagation {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "rprivate")]
    Rprivate,
    #[serde(rename = "shared")]
    Shared,
    #[serde(rename = "rshared")]
    Rshared,
    #[serde(rename = "slave")]
    Slave,
    #[serde(rename = "rslave")]
    Rslave,
}

impl Default for Propagation {
    fn default() -> Propagation {
        Self::Private
    }
}
