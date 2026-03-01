use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceMeshRegistryDocument {
    pub version: String,
    pub services: Vec<ServiceRegistration>,
    #[serde(default)]
    pub publish_ingress_policy: Option<PublishIngressPolicy>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishIngressHopRuntimeLimit {
    pub hop_name: String,
    pub configured_max_body_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishIngressPolicy {
    pub policy_owner_product: String,
    pub publish_api_contract: String,
    pub default_max_body_bytes: u64,
    pub required_hops: Vec<PublishIngressRequiredHop>,
    pub observability: PublishIngressObservability,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishIngressRequiredHop {
    pub hop_name: String,
    pub product: String,
    pub max_body_bytes_env_var: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishIngressObservability {
    pub rejection_metric_name: String,
    pub rejection_log_fields: Vec<String>,
}
