#[cfg(feature = "forbidden-api")]
pub use graphshell_endpoint::door::app_client::AppConnection;
pub use graphshell_endpoint::door::wire::{AppHostMessage, AppMessage, BrowserChallenge};

#[cfg(test)]
mod tests {
    use graphshell_endpoint::door::{app_admission::AppHello, frame::read_native_message};

    #[test]
    fn installed_hello_decodes_from_native_endian_reference_frame() {
        let body = br#"{"schema":"mere.graphshell/app-broker-hello/v2","app":"turnstone","route":"custody"}"#;
        let mut frame = (body.len() as u32).to_ne_bytes().to_vec();
        frame.extend_from_slice(body);
        let hello: AppHello = read_native_message(&mut frame.as_slice()).unwrap().unwrap();
        let request = hello.accept().unwrap();
        assert_eq!(request.route.as_str(), "custody");
    }
}
