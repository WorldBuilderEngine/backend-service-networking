use std::collections::HashSet;

use url::Url;

use crate::error::MeshRegistryError;
use crate::models::ServiceMeshRegistryDocument;

pub(crate) fn validate_registry_document(
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
