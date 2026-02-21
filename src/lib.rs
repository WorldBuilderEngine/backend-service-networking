mod constants;
mod error;
mod models;
mod registry;
mod validation;

pub use constants::{
    API_DISCOVERY_CATALOG_V1, API_DISCOVERY_DETAIL_V1, API_DISCOVERY_PLAY_SESSION_GET_V1,
    API_DISCOVERY_SCHEMA_V1, ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON,
    ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH, MVP_ANON_2D_API_CONTRACTS,
};
pub use error::MeshRegistryError;
pub use models::{ResolvedServiceTarget, ServiceMeshRegistryDocument, ServiceRegistration};
pub use registry::ServiceMeshRegistry;

#[cfg(test)]
mod tests;
