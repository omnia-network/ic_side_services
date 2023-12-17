/*
 * Flux
 *
 * This is an API documentation of calls available to be made to any Flux. <br> GET Calls are available as both query and in order as path. <br> Flux is completely open source and we encourage everyone to feel free and contribute :) <br> Further questions or support join and ask in our [discord](https://discord.io/runonflux)  # Introduction Flux possesses a 5 tier hiearchy level API. * **Public** API level - Available without any permission, does not require signing. * **User** API level - User level permission, requires signing. * **FluxTeam** API level - FluxTeam level permission (an appointed Flux Team member has access to those API calls), requires signing. * **Admin** API level - Admin level permission, requires signing. Flux owner. * **AdminAndFluxTeam** API level permission (Admin and Flux Team has access to these calls), requires signing. * **AppOwner** API level - AppOwner level permission, requires signing. App Owner. * **AppOwnerAbove** API level - AppOwnerAbove level permission (App Owner, FluxTeam, and Admin has access to these calls), requires signing.  Most calls are available via GET request with some that may require large amount of data via POST request. Websocket is currently used only for simplifying login operations and for internal Flux communication. # Getting Started with the API * **1. Install Zelcore Wallet:** The Zelcore wallet is required for signing messages. Please install this if not already installed. * **2. Create Zelcore Account:** A Zelcore wallet account is required for signing messages. Please register an account if you don't already have one. * **3. Obtain API Authentication Credentials:** Follow the Authentication section of this API documentation in order to set up your credentials for using the rest of the API. You will need to use your Zelcore wallet for parts of this process. * **4. Set Up Authentication Credentials in Header:** Set up your zelidauth credentials in your header. If you are using variables, the signature value may need to be URLencoded.
 *
 * The version of the OpenAPI document: 4.9.1
 * Contact: tadeas@runonflux.io
 * Generated by: https://openapi-generator.tech
 */

/// AppnameComposeInner : component specification



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppnameComposeInner {
    /// Name of app
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description of component
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Repotag of docker image in this format `namespace/repository:tag`
    #[serde(rename = "repotag", skip_serializing_if = "Option::is_none")]
    pub repotag: Option<String>,
    #[serde(rename = "ports", skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<i32>>,
    #[serde(rename = "domains", skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
    #[serde(rename = "environmentParameters", skip_serializing_if = "Option::is_none")]
    pub environment_parameters: Option<Vec<String>>,
    #[serde(rename = "commands", skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,
    #[serde(rename = "containerPorts", skip_serializing_if = "Option::is_none")]
    pub container_ports: Option<Vec<i32>>,
    /// Directory the data is stored and what dir is tied to the system folder
    #[serde(rename = "containerData", skip_serializing_if = "Option::is_none")]
    pub container_data: Option<String>,
    /// CPU amount requesting
    #[serde(rename = "cpu", skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f32>,
    /// Ram amount requesting in MB
    #[serde(rename = "ram", skip_serializing_if = "Option::is_none")]
    pub ram: Option<i32>,
    /// Storage space requesting in GB
    #[serde(rename = "hdd", skip_serializing_if = "Option::is_none")]
    pub hdd: Option<i32>,
    /// If set to true you will also need following key/value pairs to provide tiered info. If false do not add the following key/value pairs.
    #[serde(rename = "tiered", skip_serializing_if = "Option::is_none")]
    pub tiered: Option<bool>,
    /// CPU amount
    #[serde(rename = "cpubasic", skip_serializing_if = "Option::is_none")]
    pub cpubasic: Option<f32>,
    /// CPU amount
    #[serde(rename = "cpusuper", skip_serializing_if = "Option::is_none")]
    pub cpusuper: Option<f32>,
    /// CPU amount
    #[serde(rename = "cpubamf", skip_serializing_if = "Option::is_none")]
    pub cpubamf: Option<f32>,
    /// Ram amount in MB
    #[serde(rename = "rambasic", skip_serializing_if = "Option::is_none")]
    pub rambasic: Option<i32>,
    /// Ram amount in MB
    #[serde(rename = "ramsuper", skip_serializing_if = "Option::is_none")]
    pub ramsuper: Option<i32>,
    /// Ram amount in MB
    #[serde(rename = "rambamf", skip_serializing_if = "Option::is_none")]
    pub rambamf: Option<i32>,
    /// Storage space in GB
    #[serde(rename = "hddbasic", skip_serializing_if = "Option::is_none")]
    pub hddbasic: Option<i32>,
    /// Storage space in GB
    #[serde(rename = "hddsuper", skip_serializing_if = "Option::is_none")]
    pub hddsuper: Option<i32>,
    /// Storage space in GB
    #[serde(rename = "hddbamf", skip_serializing_if = "Option::is_none")]
    pub hddbamf: Option<i32>,
    /// Encrypted environmental accessible to selected Enterprise Nodes only.
    #[serde(rename = "secrets", skip_serializing_if = "Option::is_none")]
    pub secrets: Option<String>,
    /// Docker image authentication for private images in the format of username:apikey. This field will be encrypted and accessible to selected Enterprise Nodes only.
    #[serde(rename = "repoauth", skip_serializing_if = "Option::is_none")]
    pub repoauth: Option<String>,
}

impl AppnameComposeInner {
    /// component specification
    pub fn new() -> AppnameComposeInner {
        AppnameComposeInner {
            name: None,
            description: None,
            repotag: None,
            ports: None,
            domains: None,
            environment_parameters: None,
            commands: None,
            container_ports: None,
            container_data: None,
            cpu: None,
            ram: None,
            hdd: None,
            tiered: None,
            cpubasic: None,
            cpusuper: None,
            cpubamf: None,
            rambasic: None,
            ramsuper: None,
            rambamf: None,
            hddbasic: None,
            hddsuper: None,
            hddbamf: None,
            secrets: None,
            repoauth: None,
        }
    }
}


