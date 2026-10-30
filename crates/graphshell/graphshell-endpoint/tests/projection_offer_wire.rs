#![cfg(feature = "projection-offer")]
use graphshell_endpoint::door::projection_offer::{
    ProviderAnswer, ProviderCall, Work, WorkOperation, WorkOutcome, WorkResult,
};

#[test]
fn provider_calls_and_answers_have_stable_wire_shapes() {
    let call = ProviderCall::Next {
        completed: Some(WorkResult {
            id: 7,
            session: "viewer".into(),
            result: WorkOutcome::Closed,
        }),
    };
    let json =
        r#"{"type":"next","completed":{"id":7,"session":"viewer","result":{"type":"closed"}}}"#;
    assert_eq!(serde_json::to_string(&call).unwrap(), json);
    assert_eq!(serde_json::from_str::<ProviderCall>(json).unwrap(), call);
    let answer = ProviderAnswer::Work {
        work: Work {
            id: 8,
            session: "viewer".into(),
            operation: WorkOperation::PollNotice,
        },
    };
    let json =
        r#"{"type":"work","work":{"id":8,"session":"viewer","operation":{"type":"poll_notice"}}}"#;
    assert_eq!(serde_json::to_string(&answer).unwrap(), json);
    assert_eq!(
        serde_json::from_str::<ProviderAnswer>(json).unwrap(),
        answer
    );
    assert!(serde_json::from_str::<ProviderCall>(r#"{"type":"unknown"}"#).is_err());
    assert!(serde_json::from_str::<ProviderAnswer>(r#"{"type":"work","work":{}}"#).is_err());
}

#[test]
fn offer_and_registration_wire_fixtures_are_stable() {
    use graphshell_endpoint::door::app_admission::AppRouteId;
    let offer = ProviderCall::Offer {
        route: AppRouteId::new("sample.route").unwrap(),
        label: "Sample".into(),
    };
    let json = r#"{"type":"offer","route":"sample.route","label":"Sample"}"#;
    assert_eq!(serde_json::to_string(&offer).unwrap(), json);
    assert_eq!(serde_json::from_str::<ProviderCall>(json).unwrap(), offer);
    let registered = ProviderAnswer::Registered { max_sessions: 4 };
    let json = r#"{"type":"registered","max_sessions":4}"#;
    assert_eq!(serde_json::to_string(&registered).unwrap(), json);
    assert_eq!(
        serde_json::from_str::<ProviderAnswer>(json).unwrap(),
        registered
    );
}

#[test]
fn remaining_control_and_work_shapes_are_stable() {
    use graphshell_endpoint::door::projection_offer::ViewerContext;
    let cases = [
        (
            serde_json::to_string(&ProviderCall::Withdraw).unwrap(),
            r#"{"type":"withdraw"}"#,
        ),
        (
            serde_json::to_string(&ProviderAnswer::Idle).unwrap(),
            r#"{"type":"idle"}"#,
        ),
        (
            serde_json::to_string(&ProviderAnswer::Withdrawn).unwrap(),
            r#"{"type":"withdrawn"}"#,
        ),
        (
            serde_json::to_string(&WorkOperation::Close).unwrap(),
            r#"{"type":"close"}"#,
        ),
        (
            serde_json::to_string(&WorkOutcome::Notice { notice: None }).unwrap(),
            r#"{"type":"notice","notice":null}"#,
        ),
        (
            serde_json::to_string(&WorkOutcome::Failed {
                message: "refused".into(),
            })
            .unwrap(),
            r#"{"type":"failed","message":"refused"}"#,
        ),
    ];
    for (actual, expected) in cases {
        assert_eq!(actual, expected);
    }
    let context = ViewerContext {
        session: "s".into(),
        subject: [0; 32],
        application: None,
    };
    let encoded = serde_json::to_string(&WorkOperation::Open {
        context: context.clone(),
    })
    .unwrap();
    assert_eq!(
        serde_json::from_str::<WorkOperation>(&encoded).unwrap(),
        WorkOperation::Open { context }
    );
    assert!(
        serde_json::from_str::<WorkOperation>(
            r#"{"type":"open","context":{"session":"s","subject":[0],"application":null}}"#
        )
        .is_err()
    );
}

#[test]
fn request_and_response_payloads_preserve_chirograph_shapes() {
    use chirograph::{CarrierRequest, CarrierRequestBody, CarrierResponse, CarrierResponseBody};
    let request = WorkOperation::Request {
        request: CarrierRequest {
            id: 9,
            body: CarrierRequestBody::Discover,
        },
    };
    let json = r#"{"type":"request","request":{"id":9,"body":"Discover"}}"#;
    assert_eq!(serde_json::to_string(&request).unwrap(), json);
    assert_eq!(
        serde_json::from_str::<WorkOperation>(json).unwrap(),
        request
    );
    let response = WorkOutcome::Response {
        response: CarrierResponse {
            id: 9,
            body: Ok(CarrierResponseBody::Closed),
        },
    };
    let json = r#"{"type":"response","response":{"id":9,"body":{"Ok":"Closed"}}}"#;
    assert_eq!(serde_json::to_string(&response).unwrap(), json);
    assert_eq!(serde_json::from_str::<WorkOutcome>(json).unwrap(), response);
}
