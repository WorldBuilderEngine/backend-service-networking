use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use crate::constants::{ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON, ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH};
use crate::error::MeshRegistryError;
use crate::models::{
    PublishIngressHopRuntimeLimit, PublishIngressPolicy, PublishIngressRequiredHop, ResolvedServiceTarget, ServiceMeshRegistryDocument, ServiceRegistration,
};
use crate::validation::validate_registry_document;

#[derive(Clone, Debug)]
pub struct ServiceMeshRegistry {
    version: String,
    services: Vec<ServiceRegistration>,
    publish_ingress_policy: Option<PublishIngressPolicy>,
    api_contract_to_service_index: HashMap<String, usize>,
}

impl ServiceMeshRegistry {
    pub fn from_document(document: ServiceMeshRegistryDocument) -> Result<Self, MeshRegistryError> {
        validate_registry_document(&document)?;
        let mut api_contract_to_service_index = HashMap::<String, usize>::new();
        for (service_index, service) in document.services.iter().enumerate() {
            for api_contract in &service.api_contracts {
                api_contract_to_service_index.insert(api_contract.clone(), service_index);
            }
        }

        Ok(Self {
            version: document.version,
            services: document.services,
            publish_ingress_policy: document.publish_ingress_policy,
            api_contract_to_service_index,
        })
    }

    pub fn from_json_str(registry_json: &str) -> Result<Self, MeshRegistryError> {
        let document =
            serde_json::from_str::<ServiceMeshRegistryDocument>(registry_json).map_err(|decode_error| MeshRegistryError::Decode(decode_error.to_string()))?;
        Self::from_document(document)
    }

    pub fn from_file_path(registry_path: impl AsRef<Path>) -> Result<Self, MeshRegistryError> {
        let registry_source = fs::read_to_string(registry_path.as_ref()).map_err(|io_error| MeshRegistryError::Io(io_error.to_string()))?;
        Self::from_json_str(&registry_source)
    }

    pub fn single_service(
        version: impl Into<String>,
        service_name: impl Into<String>,
        base_url: impl Into<String>,
        api_contracts: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, MeshRegistryError> {
        let document = ServiceMeshRegistryDocument {
            version: version.into(),
            services: vec![ServiceRegistration {
                service_name: service_name.into(),
                base_url: base_url.into(),
                api_contracts: api_contracts.into_iter().map(Into::into).collect(),
            }],
            publish_ingress_policy: None,
        };
        Self::from_document(document)
    }

    pub fn from_environment() -> Result<Option<Self>, MeshRegistryError> {
        if let Ok(registry_json_source) = env::var(ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON) {
            if !registry_json_source.trim().is_empty() {
                return Ok(Some(Self::from_json_str(registry_json_source.as_str())?));
            }
        }

        if let Ok(registry_path_source) = env::var(ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH) {
            if !registry_path_source.trim().is_empty() {
                return Ok(Some(Self::from_file_path(registry_path_source)?));
            }
        }

        Ok(None)
    }

    pub fn from_environment_or_single_service(
        version: impl Into<String>,
        service_name: impl Into<String>,
        base_url: impl Into<String>,
        api_contracts: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, MeshRegistryError> {
        if let Some(registry) = Self::from_environment()? {
            return Ok(registry);
        }
        Self::single_service(version, service_name, base_url, api_contracts)
    }

    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    pub fn resolve_api_contract(
        &self,
        api_contract: &str,
    ) -> Result<ResolvedServiceTarget, MeshRegistryError> {
        let normalized_api_contract = api_contract.trim();
        let Some(service_index) = self.api_contract_to_service_index.get(normalized_api_contract) else {
            return Err(MeshRegistryError::UnknownApiContract(normalized_api_contract.to_string()));
        };
        let service = &self.services[*service_index];
        Ok(ResolvedServiceTarget {
            service_name: service.service_name.clone(),
            base_url: service.base_url.clone(),
            api_contract: normalized_api_contract.to_string(),
        })
    }

    pub fn ensure_contracts_registered(
        &self,
        required_api_contracts: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<(), MeshRegistryError> {
        let mut missing_api_contracts = Vec::<String>::new();
        for required_api_contract in required_api_contracts {
            let normalized_api_contract = required_api_contract.as_ref().trim();
            if normalized_api_contract.is_empty() {
                return Err(MeshRegistryError::InvalidDocument(
                    "required api contract list contains an empty value".to_string(),
                ));
            }
            if !self
                .api_contract_to_service_index
                .contains_key(normalized_api_contract)
            {
                missing_api_contracts.push(normalized_api_contract.to_string());
            }
        }

        if missing_api_contracts.is_empty() {
            return Ok(());
        }

        missing_api_contracts.sort();
        missing_api_contracts.dedup();
        Err(MeshRegistryError::MissingRequiredApiContracts(missing_api_contracts))
    }

    pub fn publish_ingress_policy(&self) -> Option<&PublishIngressPolicy> {
        self.document_publish_ingress_policy()
    }

    pub fn ensure_publish_ingress_hop_limit_from_environment(
        &self,
        hop_name: &str,
    ) -> Result<PublishIngressHopRuntimeLimit, MeshRegistryError> {
        let required_hop = self.resolve_publish_ingress_required_hop(hop_name)?;
        let env_var_value = env::var(required_hop.max_body_bytes_env_var.as_str()).map_err(|_| MeshRegistryError::MissingPublishIngressHopLimit {
            hop_name: required_hop.hop_name.clone(),
            env_var: required_hop.max_body_bytes_env_var.clone(),
        })?;
        let parsed_limit = env_var_value
            .parse::<u64>()
            .map_err(|_| MeshRegistryError::InvalidPublishIngressHopLimit {
                hop_name: required_hop.hop_name.clone(),
                env_var: required_hop.max_body_bytes_env_var.clone(),
                value: env_var_value.clone(),
            })?;
        self.ensure_publish_ingress_hop_limit(required_hop.hop_name.as_str(), parsed_limit)?;
        Ok(PublishIngressHopRuntimeLimit {
            hop_name: required_hop.hop_name.clone(),
            configured_max_body_bytes: parsed_limit,
        })
    }

    pub fn ensure_publish_ingress_hop_limit(
        &self,
        hop_name: &str,
        configured_max_body_bytes: u64,
    ) -> Result<(), MeshRegistryError> {
        self.resolve_publish_ingress_required_hop(hop_name)?;
        let publish_ingress_policy = self.require_publish_ingress_policy()?;
        if configured_max_body_bytes < publish_ingress_policy.default_max_body_bytes {
            return Err(MeshRegistryError::PublishIngressHopLimitTooLow {
                hop_name: hop_name.trim().to_string(),
                configured_max_body_bytes,
                required_min_body_bytes: publish_ingress_policy.default_max_body_bytes,
            });
        }
        Ok(())
    }

    pub fn ensure_publish_ingress_all_hops_conform(
        &self,
        configured_hop_limits: impl IntoIterator<Item = PublishIngressHopRuntimeLimit>,
    ) -> Result<(), MeshRegistryError> {
        let publish_ingress_policy = self.require_publish_ingress_policy()?;
        let mut configured_hop_limits_by_name = HashMap::<String, u64>::new();
        for configured_hop_limit in configured_hop_limits {
            configured_hop_limits_by_name.insert(configured_hop_limit.hop_name.trim().to_string(), configured_hop_limit.configured_max_body_bytes);
        }

        for required_hop in &publish_ingress_policy.required_hops {
            let Some(configured_max_body_bytes) = configured_hop_limits_by_name.get(required_hop.hop_name.trim()) else {
                return Err(MeshRegistryError::MissingPublishIngressHopLimit {
                    hop_name: required_hop.hop_name.clone(),
                    env_var: required_hop.max_body_bytes_env_var.clone(),
                });
            };
            self.ensure_publish_ingress_hop_limit(required_hop.hop_name.as_str(), *configured_max_body_bytes)?;
        }

        Ok(())
    }

    fn document_publish_ingress_policy(&self) -> Option<&PublishIngressPolicy> {
        self.publish_ingress_policy.as_ref()
    }

    fn require_publish_ingress_policy(&self) -> Result<&PublishIngressPolicy, MeshRegistryError> {
        self.document_publish_ingress_policy()
            .ok_or(MeshRegistryError::MissingPublishIngressPolicy)
    }

    fn resolve_publish_ingress_required_hop(
        &self,
        hop_name: &str,
    ) -> Result<&PublishIngressRequiredHop, MeshRegistryError> {
        let normalized_hop_name = hop_name.trim();
        let publish_ingress_policy = self.require_publish_ingress_policy()?;
        publish_ingress_policy
            .required_hops
            .iter()
            .find(|required_hop| required_hop.hop_name.trim() == normalized_hop_name)
            .ok_or_else(|| MeshRegistryError::MissingPublishIngressHop(normalized_hop_name.to_string()))
    }
}
