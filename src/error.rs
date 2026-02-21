use std::fmt;

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
