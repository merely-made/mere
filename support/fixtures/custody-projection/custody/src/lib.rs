#[cfg(feature = "forbidden-api")]
pub use graphshell_endpoint::ProjectionCatalog;
pub use graphshell_endpoint::door::custody_client::{BlockingCustodyClient, CustodyClient};

#[cfg(test)]
mod tests {
    use graphshell_endpoint::door::{custody::CustodyRefusal, custody_client::CustodyClientError};

    #[test]
    fn legacy_refusal_is_pending_without_a_projection_api() {
        let refusal: CustodyRefusal = serde_json::from_str(r#"{"refusal":"locked"}"#).unwrap();
        assert!(CustodyClientError::Refused(refusal).is_pending());
        let refusal: CustodyRefusal =
            serde_json::from_str(r#"{"refusal":"not_releasable"}"#).unwrap();
        assert!(!CustodyClientError::Refused(refusal).is_pending());
    }
}
