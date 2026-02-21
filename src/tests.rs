use std::env;
use std::fs;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    API_DISCOVERY_CATALOG_V1, API_DISCOVERY_DETAIL_V1, API_DISCOVERY_PLAY_SESSION_GET_V1,
    API_DISCOVERY_PUBLISH_CREATE_V1,
    API_DISCOVERY_SCHEMA_V1, ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON,
    ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH, MVP_ANON_2D_GATEWAY_API_CONTRACTS,
    MVP_ANON_2D_READ_API_CONTRACTS, MeshRegistryError, ServiceMeshRegistry,
    ServiceMeshRegistryDocument, ServiceRegistration,
};

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
        MVP_ANON_2D_GATEWAY_API_CONTRACTS,
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
                api_contracts: vec![API_DISCOVERY_DETAIL_V1.to_string()],
            },
            ServiceRegistration {
                service_name: "backend-data-center-b".to_string(),
                base_url: "http://127.0.0.1:8789".to_string(),
                api_contracts: vec![API_DISCOVERY_DETAIL_V1.to_string()],
            },
        ],
    };

    let error = ServiceMeshRegistry::from_document(registry_document).unwrap_err();
    assert_eq!(
        error,
        MeshRegistryError::InvalidDocument(
            "api contract 'worldbuilder.discovery.detail.v1' is registered by multiple services"
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
                    "worldbuilder.discovery.detail.v1"
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
                "api_contracts": ["worldbuilder.discovery.detail.v1"]
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
        .resolve_api_contract(API_DISCOVERY_DETAIL_V1)
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
        MVP_ANON_2D_GATEWAY_API_CONTRACTS,
    )
    .unwrap();

    registry
        .ensure_contracts_registered(MVP_ANON_2D_GATEWAY_API_CONTRACTS)
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
        .ensure_contracts_registered(MVP_ANON_2D_GATEWAY_API_CONTRACTS)
        .unwrap_err();
    assert_eq!(
        error,
        MeshRegistryError::MissingRequiredApiContracts(vec![
            API_DISCOVERY_DETAIL_V1.to_string(),
            API_DISCOVERY_PLAY_SESSION_GET_V1.to_string(),
            API_DISCOVERY_PUBLISH_CREATE_V1.to_string(),
            API_DISCOVERY_SCHEMA_V1.to_string(),
        ])
    );
}

#[test]
fn mvp_read_contracts_exclude_publish_contract() {
    assert!(!MVP_ANON_2D_READ_API_CONTRACTS.contains(&API_DISCOVERY_PUBLISH_CREATE_V1));
    assert_eq!(MVP_ANON_2D_READ_API_CONTRACTS.len(), 4);
}
