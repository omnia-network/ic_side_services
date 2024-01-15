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
pub struct AppInspect200ResponseData {
    /// ID of the container
    #[serde(rename = "Id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Date and time when container was created
    #[serde(rename = "Created", skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Script path
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "Args", skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(rename = "State", skip_serializing_if = "Option::is_none")]
    pub state: Option<Box<crate::models::AppInspect200ResponseDataState>>,
    /// Image id
    #[serde(rename = "Image", skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Path of containers resolv.conf
    #[serde(rename = "ResolvConfPath", skip_serializing_if = "Option::is_none")]
    pub resolv_conf_path: Option<String>,
    /// Path of containers hostname
    #[serde(rename = "HostnamePath", skip_serializing_if = "Option::is_none")]
    pub hostname_path: Option<String>,
    /// Path of hosts
    #[serde(rename = "HostsPath", skip_serializing_if = "Option::is_none")]
    pub hosts_path: Option<String>,
    /// Path of log
    #[serde(rename = "LogPath", skip_serializing_if = "Option::is_none")]
    pub log_path: Option<String>,
    /// Container name
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Number of restarts
    #[serde(rename = "RestartCount", skip_serializing_if = "Option::is_none")]
    pub restart_count: Option<i32>,
    /// Storage driver type
    #[serde(rename = "Driver", skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    /// OS type
    #[serde(rename = "Platform", skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Container mount label
    #[serde(rename = "MountLabel", skip_serializing_if = "Option::is_none")]
    pub mount_label: Option<String>,
    /// Container process label
    #[serde(rename = "ProcessLabel", skip_serializing_if = "Option::is_none")]
    pub process_label: Option<String>,
    /// Application Armor profile
    #[serde(rename = "AppArmorProfile", skip_serializing_if = "Option::is_none")]
    pub app_armor_profile: Option<String>,
    /// Exec ids of commands
    #[serde(rename = "ExecIDs", skip_serializing_if = "Option::is_none")]
    pub exec_ids: Option<String>,
    #[serde(rename = "HostConfig", skip_serializing_if = "Option::is_none")]
    pub host_config: Option<Box<crate::models::AppInspect200ResponseDataHostConfig>>,
    #[serde(rename = "GraphDriver", skip_serializing_if = "Option::is_none")]
    pub graph_driver: Option<Box<crate::models::AppInspect200ResponseDataGraphDriver>>,
    #[serde(rename = "Mounts", skip_serializing_if = "Option::is_none")]
    pub mounts: Option<Vec<crate::models::MountsInner>>,
    #[serde(rename = "Config", skip_serializing_if = "Option::is_none")]
    pub config: Option<Box<crate::models::AppInspect200ResponseDataConfig>>,
    #[serde(rename = "NetworkSettings", skip_serializing_if = "Option::is_none")]
    pub network_settings: Option<Box<crate::models::AppInspect200ResponseDataNetworkSettings>>,
}

impl AppInspect200ResponseData {
    pub fn new() -> AppInspect200ResponseData {
        AppInspect200ResponseData {
            id: None,
            created: None,
            path: None,
            args: None,
            state: None,
            image: None,
            resolv_conf_path: None,
            hostname_path: None,
            hosts_path: None,
            log_path: None,
            name: None,
            restart_count: None,
            driver: None,
            platform: None,
            mount_label: None,
            process_label: None,
            app_armor_profile: None,
            exec_ids: None,
            host_config: None,
            graph_driver: None,
            mounts: None,
            config: None,
            network_settings: None,
        }
    }
}

