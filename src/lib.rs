use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use url::Url;

pub const API_DISCOVERY_CATALOG_V1: &str = "worldbuilder.discovery.catalog.v1";
pub const API_DISCOVERY_HOME_V1: &str = "worldbuilder.discovery.home.v1";
pub const API_DISCOVERY_DETAIL_V1: &str = "worldbuilder.discovery.detail.v1";
pub const API_DISCOVERY_SCHEMA_V1: &str = "worldbuilder.discovery.schema.v1";
pub const API_DISCOVERY_PLAY_SESSION_GET_V1: &str = "worldbuilder.discovery.play-session.get.v1";
pub const API_DISCOVERY_PLAY_SESSION_CREATE_V1: &str =
    "worldbuilder.discovery.play-session.create.v1";

pub const ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH: &str =
    "WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH";
pub const ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON: &str =
    "WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON";

pub const MVP_ANON_2D_API_CONTRACTS: [&str; 6] = [
    API_DISCOVERY_CATALOG_V1,
    API_DISCOVERY_HOME_V1,
    API_DISCOVERY_DETAIL_V1,
    API_DISCOVERY_SCHEMA_V1,
    API_DISCOVERY_PLAY_SESSION_GET_V1,
    API_DISCOVERY_PLAY_SESSION_CREATE_V1,
];

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

#[derive(Debug, PartialEq, Eq)]
pub enum MeshRegistryError {
    InvalidDocument(String),
    UnknownApiContract(String),
    MissingRequiredApiContracts(Vec<String>),
    Decode(String),
    Io(String),
}

impl fmt::Display for MeshRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeshRegistryError::InvalidDocument(message) => {
                write!(formatter, "invalid service mesh registry: {}.", message)
            }
            MeshRegistryError::UnknownApiContract(api_contract) => {
                write!(
                    formatter,
                    "service mesh api contract '{}' is not registered.",
                    api_contract
                )
            }
            MeshRegistryError::MissingRequiredApiContracts(missing_api_contracts) => {
                write!(
                    formatter,
                    "service mesh registry is missing required api contracts: {}.",
                    missing_api_contracts.join(", ")
                )
            }
            MeshRegistryError::Decode(message) => write!(
                formatter,
                "failed to decode service mesh registry document: {}.",
                message
            ),
            MeshRegistryError::Io(message) => {
                write!(
                    formatter,
                    "failed to read service mesh registry source: {}.",
                    message
                )
            }
        }
    }
}

impl std::error::Error for MeshRegistryError {}

#[derive(Clone, Debug)]
pub struct ServiceMeshRegistry {
    version: String,
    services: Vec<ServiceRegistration>,
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
            api_contract_to_service_index,
        })
    }

    pub fn from_json_str(registry_json: &str) -> Result<Self, MeshRegistryError> {
        let document = serde_json::from_str::<ServiceMeshRegistryDocument>(registry_json)
            .map_err(|decode_error| MeshRegistryError::Decode(decode_error.to_string()))?;
        Self::from_document(document)
    }

    pub fn from_file_path(registry_path: impl AsRef<Path>) -> Result<Self, MeshRegistryError> {
        let registry_source = fs::read_to_string(registry_path.as_ref())
            .map_err(|io_error| MeshRegistryError::Io(io_error.to_string()))?;
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
        let Some(service_index) = self
            .api_contract_to_service_index
            .get(normalized_api_contract)
        else {
            return Err(MeshRegistryError::UnknownApiContract(
                normalized_api_contract.to_string(),
            ));
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
        Err(MeshRegistryError::MissingRequiredApiContracts(
            missing_api_contracts,
        ))
    }
}

fn validate_registry_document(
    document: &ServiceMeshRegistryDocument,
) -> Result<(), MeshRegistryError> {
    if document.version.trim().is_empty() {
        return Err(MeshRegistryError::InvalidDocument(
            "version must not be empty".to_string(),
        ));
    }
    if document.services.is_empty() {
        return Err(MeshRegistryError::InvalidDocument(
            "at least one service registration is required".to_string(),
        ));
    }

    let mut service_names = HashSet::<String>::new();
    let mut api_contracts = HashSet::<String>::new();

    for service in &document.services {
        let service_name = service.service_name.trim();
        if service_name.is_empty() {
            return Err(MeshRegistryError::InvalidDocument(
                "service_name must not be empty".to_string(),
            ));
        }
        if !service_names.insert(service_name.to_string()) {
            return Err(MeshRegistryError::InvalidDocument(format!(
                "service_name '{}' is duplicated",
                service_name
            )));
        }

        let parsed_base_url = Url::parse(service.base_url.trim()).map_err(|parse_error| {
            MeshRegistryError::InvalidDocument(format!(
                "service '{}' base_url '{}' is invalid: {}",
                service_name, service.base_url, parse_error
            ))
        })?;
        if parsed_base_url.host_str().is_none() {
            return Err(MeshRegistryError::InvalidDocument(format!(
                "service '{}' base_url '{}' must include a host",
                service_name, service.base_url
            )));
        }
        if service.api_contracts.is_empty() {
            return Err(MeshRegistryError::InvalidDocument(format!(
                "service '{}' must register at least one api contract",
                service_name
            )));
        }

        for api_contract in &service.api_contracts {
            let normalized_api_contract = api_contract.trim();
            if normalized_api_contract.is_empty() {
                return Err(MeshRegistryError::InvalidDocument(format!(
                    "service '{}' has an empty api contract entry",
                    service_name
                )));
            }
            if !api_contracts.insert(normalized_api_contract.to_string()) {
                return Err(MeshRegistryError::InvalidDocument(format!(
                    "api contract '{}' is registered by multiple services",
                    normalized_api_contract
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn environment_lock() -> &'static Mutex<()> {
        static ENVIRONMENT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENVIRONMENT_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn clear_registry_environment() {
        unsafe {
            env::remove_var(ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON);
            env::remove_var(ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH);
        }
    }

    fn set_env_var(key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
    }

    #[test]
    fn resolves_contract_to_registered_service() {
        let registry = ServiceMeshRegistry::single_service(
            "2026-02-21",
            "backend-data-center",
            "http://127.0.0.1:8787",
            MVP_ANON_2D_API_CONTRACTS,
        )
        .unwrap();

        let resolved_target = registry
            .resolve_api_contract(API_DISCOVERY_SCHEMA_V1)
            .unwrap();

        assert_eq!(resolved_target.service_name, "backend-data-center");
        assert_eq!(resolved_target.base_url, "http://127.0.0.1:8787");
        assert_eq!(resolved_target.api_contract, API_DISCOVERY_SCHEMA_V1);
    }

    #[test]
    fn rejects_duplicate_api_contract_across_services() {
        let registry_document = ServiceMeshRegistryDocument {
            version: "2026-02-21".to_string(),
            services: vec![
                ServiceRegistration {
                    service_name: "backend-data-center-a".to_string(),
                    base_url: "http://127.0.0.1:8787".to_string(),
                    api_contracts: vec![API_DISCOVERY_HOME_V1.to_string()],
                },
                ServiceRegistration {
                    service_name: "backend-data-center-b".to_string(),
                    base_url: "http://127.0.0.1:8789".to_string(),
                    api_contracts: vec![API_DISCOVERY_HOME_V1.to_string()],
                },
            ],
        };

        let error = ServiceMeshRegistry::from_document(registry_document).unwrap_err();
        assert_eq!(
            error,
            MeshRegistryError::InvalidDocument(
                "api contract 'worldbuilder.discovery.home.v1' is registered by multiple services"
                    .to_string()
            )
        );
    }

    #[test]
    fn resolves_from_json_document() {
        let registry_json = r#"{
            "version": "2026-02-21",
            "services": [
                {
                    "service_name": "backend-data-center",
                    "base_url": "http://127.0.0.1:8787",
                    "api_contracts": [
                        "worldbuilder.discovery.catalog.v1",
                        "worldbuilder.discovery.home.v1"
                    ]
                }
            ]
        }"#;

        let registry = ServiceMeshRegistry::from_json_str(registry_json).unwrap();
        let resolved_target = registry
            .resolve_api_contract(API_DISCOVERY_CATALOG_V1)
            .unwrap();

        assert_eq!(registry.version(), "2026-02-21");
        assert_eq!(resolved_target.service_name, "backend-data-center");
    }

    #[test]
    fn returns_error_for_unknown_contract() {
        let _lock = environment_lock().lock().unwrap();
        clear_registry_environment();
        let registry = ServiceMeshRegistry::single_service(
            "2026-02-21",
            "backend-data-center",
            "http://127.0.0.1:8787",
            [API_DISCOVERY_CATALOG_V1],
        )
        .unwrap();

        let error = registry
            .resolve_api_contract(API_DISCOVERY_DETAIL_V1)
            .unwrap_err();
        assert_eq!(
            error,
            MeshRegistryError::UnknownApiContract(API_DISCOVERY_DETAIL_V1.to_string())
        );
    }

    #[test]
    fn loads_registry_from_environment_json() {
        let _lock = environment_lock().lock().unwrap();
        clear_registry_environment();
        set_env_var(
            ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON,
            r#"{
                "version": "2026-02-21",
                "services": [
                    {
                        "service_name": "backend-data-center",
                        "base_url": "http://127.0.0.1:8787",
                        "api_contracts": ["worldbuilder.discovery.catalog.v1"]
                    }
                ]
            }"#,
        );

        let registry = ServiceMeshRegistry::from_environment()
            .unwrap()
            .expect("expected registry");
        let resolved_target = registry
            .resolve_api_contract(API_DISCOVERY_CATALOG_V1)
            .unwrap();
        assert_eq!(resolved_target.service_name, "backend-data-center");
    }

    #[test]
    fn loads_registry_from_environment_path_when_json_is_not_set() {
        let _lock = environment_lock().lock().unwrap();
        clear_registry_environment();
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let registry_path = env::temp_dir().join(format!(
            "backend-service-networking-registry-{}.json",
            unique_suffix
        ));
        let registry_json = r#"{
            "version": "2026-02-21",
            "services": [
                {
                    "service_name": "backend-data-center",
                    "base_url": "http://127.0.0.1:8787",
                    "api_contracts": ["worldbuilder.discovery.home.v1"]
                }
            ]
        }"#;
        fs::write(&registry_path, registry_json).expect("failed to write temp registry");
        set_env_var(
            ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH,
            registry_path.to_string_lossy().as_ref(),
        );

        let registry = ServiceMeshRegistry::from_environment()
            .unwrap()
            .expect("expected registry");
        let resolved_target = registry
            .resolve_api_contract(API_DISCOVERY_HOME_V1)
            .unwrap();
        assert_eq!(resolved_target.service_name, "backend-data-center");

        fs::remove_file(registry_path).ok();
    }

    #[test]
    fn falls_back_to_single_service_when_environment_is_empty() {
        let _lock = environment_lock().lock().unwrap();
        clear_registry_environment();
        let registry = ServiceMeshRegistry::from_environment_or_single_service(
            "2026-02-21",
            "backend-data-center",
            "http://127.0.0.1:8787",
            [API_DISCOVERY_SCHEMA_V1],
        )
        .unwrap();

        let resolved_target = registry
            .resolve_api_contract(API_DISCOVERY_SCHEMA_V1)
            .unwrap();
        assert_eq!(resolved_target.service_name, "backend-data-center");
    }

    #[test]
    fn validates_required_contracts_for_mvp() {
        let registry = ServiceMeshRegistry::single_service(
            "2026-02-21",
            "backend-data-center",
            "http://127.0.0.1:8787",
            MVP_ANON_2D_API_CONTRACTS,
        )
        .unwrap();

        registry
            .ensure_contracts_registered(MVP_ANON_2D_API_CONTRACTS)
            .unwrap();
    }

    #[test]
    fn returns_missing_required_contracts_when_registry_is_incomplete() {
        let registry = ServiceMeshRegistry::single_service(
            "2026-02-21",
            "backend-data-center",
            "http://127.0.0.1:8787",
            [API_DISCOVERY_CATALOG_V1],
        )
        .unwrap();

        let error = registry
            .ensure_contracts_registered(MVP_ANON_2D_API_CONTRACTS)
            .unwrap_err();
        assert_eq!(
            error,
            MeshRegistryError::MissingRequiredApiContracts(vec![
                API_DISCOVERY_DETAIL_V1.to_string(),
                API_DISCOVERY_HOME_V1.to_string(),
                API_DISCOVERY_PLAY_SESSION_CREATE_V1.to_string(),
                API_DISCOVERY_PLAY_SESSION_GET_V1.to_string(),
                API_DISCOVERY_SCHEMA_V1.to_string(),
            ])
        );
    }
}
