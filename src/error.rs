use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum MeshRegistryError {
    InvalidDocument(String),
    UnknownApiContract(String),
    MissingRequiredApiContracts(Vec<String>),
    MissingPublishIngressPolicy,
    MissingPublishIngressHop(String),
    MissingPublishIngressHopLimit {
        hop_name: String,
        env_var: String,
    },
    InvalidPublishIngressHopLimit {
        hop_name: String,
        env_var: String,
        value: String,
    },
    PublishIngressHopLimitTooLow {
        hop_name: String,
        configured_max_body_bytes: u64,
        required_min_body_bytes: u64,
    },
    Decode(String),
    Io(String),
}

impl fmt::Display for MeshRegistryError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            MeshRegistryError::InvalidDocument(message) => {
                write!(formatter, "invalid service mesh registry: {}.", message)
            }
            MeshRegistryError::UnknownApiContract(api_contract) => {
                write!(formatter, "service mesh api contract '{}' is not registered.", api_contract)
            }
            MeshRegistryError::MissingRequiredApiContracts(missing_api_contracts) => {
                write!(
                    formatter,
                    "service mesh registry is missing required api contracts: {}.",
                    missing_api_contracts.join(", ")
                )
            }
            MeshRegistryError::MissingPublishIngressPolicy => write!(formatter, "service mesh registry is missing publish ingress policy."),
            MeshRegistryError::MissingPublishIngressHop(hop_name) => write!(formatter, "publish ingress policy does not define required hop '{}'.", hop_name),
            MeshRegistryError::MissingPublishIngressHopLimit { hop_name, env_var } => write!(
                formatter,
                "publish ingress hop '{}' is missing configured body limit env '{}'.",
                hop_name, env_var
            ),
            MeshRegistryError::InvalidPublishIngressHopLimit { hop_name, env_var, value } => write!(
                formatter,
                "publish ingress hop '{}' env '{}' must be a positive integer byte value, got '{}'.",
                hop_name, env_var, value
            ),
            MeshRegistryError::PublishIngressHopLimitTooLow {
                hop_name,
                configured_max_body_bytes,
                required_min_body_bytes,
            } => write!(
                formatter,
                "publish ingress hop '{}' max body {} bytes is below required {} bytes.",
                hop_name, configured_max_body_bytes, required_min_body_bytes
            ),
            MeshRegistryError::Decode(message) => write!(formatter, "failed to decode service mesh registry document: {}.", message),
            MeshRegistryError::Io(message) => {
                write!(formatter, "failed to read service mesh registry source: {}.", message)
            }
        }
    }
}

impl std::error::Error for MeshRegistryError {}
