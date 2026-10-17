#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ViewerSubsystemCapabilities {
    pub accessibility: CapabilityDeclaration,
    pub security: CapabilityDeclaration,
    pub storage: CapabilityDeclaration,
    pub history: CapabilityDeclaration,
}

impl ViewerSubsystemCapabilities {
    pub fn full() -> Self {
        Self {
            accessibility: CapabilityDeclaration::full(),
            security: CapabilityDeclaration::full(),
            storage: CapabilityDeclaration::full(),
            history: CapabilityDeclaration::full(),
        }
    }
}

