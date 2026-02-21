use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceMeshRegistryDocument {
    pub version: String,
    pub services: Vec<ServiceRegistration>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_name: String,
    pub base_url: String,
    pub api_contracts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedServiceTarget {
    pub service_name: String,
    pub base_url: String,
    pub api_contract: String,
}
