pub const API_DISCOVERY_CATALOG_V1: &str = "worldbuilder.discovery.catalog.v1";
pub const API_DISCOVERY_HOME_FEED_V1: &str = "worldbuilder.discovery.home_feed.v1";
pub const API_DISCOVERY_DETAIL_V1: &str = "worldbuilder.discovery.detail.v1";
pub const API_DISCOVERY_SCHEMA_V1: &str = "worldbuilder.discovery.schema.v1";
pub const API_DISCOVERY_PLAY_SESSION_GET_V1: &str = "worldbuilder.discovery.play-session.get.v1";
pub const API_DISCOVERY_PUBLISH_CREATE_V1: &str = "worldbuilder.discovery.publish.create.v1";
pub const API_AUTH_REGISTER_V1: &str = "worldbuilder.auth.register.v1";
pub const API_AUTH_LOGIN_V1: &str = "worldbuilder.auth.login.v1";
pub const API_AUTH_REFRESH_V1: &str = "worldbuilder.auth.refresh.v1";
pub const API_AUTH_GUEST_UPGRADE_V1: &str = "worldbuilder.auth.guest-upgrade.v1";

pub const ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH: &str = "WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH";
pub const ENV_WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON: &str = "WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON";

pub const MVP_ANON_2D_READ_API_CONTRACTS: [&str; 5] = [
    API_DISCOVERY_HOME_FEED_V1,
    API_DISCOVERY_CATALOG_V1,
    API_DISCOVERY_DETAIL_V1,
    API_DISCOVERY_SCHEMA_V1,
    API_DISCOVERY_PLAY_SESSION_GET_V1,
];

pub const MVP_ANON_2D_GATEWAY_API_CONTRACTS: [&str; 10] = [
    API_DISCOVERY_HOME_FEED_V1,
    API_DISCOVERY_CATALOG_V1,
    API_DISCOVERY_DETAIL_V1,
    API_DISCOVERY_SCHEMA_V1,
    API_DISCOVERY_PLAY_SESSION_GET_V1,
    API_DISCOVERY_PUBLISH_CREATE_V1,
    API_AUTH_REGISTER_V1,
    API_AUTH_LOGIN_V1,
    API_AUTH_REFRESH_V1,
    API_AUTH_GUEST_UPGRADE_V1,
];
