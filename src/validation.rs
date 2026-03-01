use std::collections::HashSet;

use url::Url;

use crate::error::MeshRegistryError;
use crate::models::{PublishIngressPolicy, ServiceMeshRegistryDocument};

pub(crate) fn validate_registry_document(document: &ServiceMeshRegistryDocument) -> Result<(), MeshRegistryError> {
    if document.version.trim().is_empty() {
        return Err(MeshRegistryError::InvalidDocument("version must not be empty".to_string()));
    }
    if document.services.is_empty() {
        return Err(MeshRegistryError::InvalidDocument("at least one service registration is required".to_string()));
    }

    let mut service_names = HashSet::<String>::new();
    let mut api_contracts = HashSet::<String>::new();

    for service in &document.services {
        let service_name = service.service_name.trim();
        if service_name.is_empty() {
            return Err(MeshRegistryError::InvalidDocument("service_name must not be empty".to_string()));
        }
        if !service_names.insert(service_name.to_string()) {
            return Err(MeshRegistryError::InvalidDocument(format!("service_name '{}' is duplicated", service_name)));
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

    if let Some(publish_ingress_policy) = &document.publish_ingress_policy {
        validate_publish_ingress_policy(publish_ingress_policy)?;
    }

    Ok(())
}

fn validate_publish_ingress_policy(publish_ingress_policy: &PublishIngressPolicy) -> Result<(), MeshRegistryError> {
    if publish_ingress_policy.policy_owner_product.trim().is_empty() {
        return Err(MeshRegistryError::InvalidDocument(
            "publish_ingress_policy.policy_owner_product must not be empty".to_string(),
        ));
    }
    if publish_ingress_policy.publish_api_contract.trim().is_empty() {
        return Err(MeshRegistryError::InvalidDocument(
            "publish_ingress_policy.publish_api_contract must not be empty".to_string(),
        ));
    }
    if publish_ingress_policy.default_max_body_bytes == 0 {
        return Err(MeshRegistryError::InvalidDocument(
            "publish_ingress_policy.default_max_body_bytes must be greater than zero".to_string(),
        ));
    }
    if publish_ingress_policy.required_hops.is_empty() {
        return Err(MeshRegistryError::InvalidDocument(
            "publish_ingress_policy.required_hops must include at least one hop".to_string(),
        ));
    }
    if publish_ingress_policy
        .observability
        .rejection_metric_name
        .trim()
        .is_empty()
    {
        return Err(MeshRegistryError::InvalidDocument(
            "publish_ingress_policy.observability.rejection_metric_name must not be empty".to_string(),
        ));
    }
    if publish_ingress_policy
        .observability
        .rejection_log_fields
        .is_empty()
    {
        return Err(MeshRegistryError::InvalidDocument(
            "publish_ingress_policy.observability.rejection_log_fields must include at least one field".to_string(),
        ));
    }

    let mut hop_names = HashSet::<String>::new();
    let mut hop_env_var_names = HashSet::<String>::new();
    for required_hop in &publish_ingress_policy.required_hops {
        let hop_name = required_hop.hop_name.trim();
        if hop_name.is_empty() {
            return Err(MeshRegistryError::InvalidDocument(
                "publish_ingress_policy.required_hops[].hop_name must not be empty".to_string(),
            ));
        }
        if !hop_names.insert(hop_name.to_string()) {
            return Err(MeshRegistryError::InvalidDocument(format!(
                "publish_ingress_policy.required_hops contains duplicate hop '{}'",
                hop_name
            )));
        }

        if required_hop.product.trim().is_empty() {
            return Err(MeshRegistryError::InvalidDocument(format!(
                "publish_ingress_policy.required_hops['{}'].product must not be empty",
                hop_name
            )));
        }

        let max_body_bytes_env_var = required_hop.max_body_bytes_env_var.trim();
        if max_body_bytes_env_var.is_empty() {
            return Err(MeshRegistryError::InvalidDocument(format!(
                "publish_ingress_policy.required_hops['{}'].max_body_bytes_env_var must not be empty",
                hop_name
            )));
        }
        if !hop_env_var_names.insert(max_body_bytes_env_var.to_string()) {
            return Err(MeshRegistryError::InvalidDocument(format!(
                "publish_ingress_policy.required_hops uses duplicate max_body_bytes_env_var '{}'",
                max_body_bytes_env_var
            )));
        }
    }

    for rejection_log_field in &publish_ingress_policy.observability.rejection_log_fields {
        if rejection_log_field.trim().is_empty() {
            return Err(MeshRegistryError::InvalidDocument(
                "publish_ingress_policy.observability.rejection_log_fields contains an empty field".to_string(),
            ));
        }
    }

    Ok(())
}
