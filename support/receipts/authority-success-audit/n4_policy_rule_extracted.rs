fn rule(access: ServiceAccess, max_sessions: Option<u32>) -> ServiceRule {
    ServiceRule::new(access, "mere.network", ["connect"], false, max_sessions)
}
