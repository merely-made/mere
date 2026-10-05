use cubecl::server::Handle;

// Bind one input only when both handles address the same view and service.
// Rulings 355/377: use the public allocation identity, preserving all prior
// view comparisons and adding the complete service identity.
pub(super) fn same_view(a: &Handle, b: &Handle) -> bool {
    a.memory.descriptor().id == b.memory.descriptor().id
        && a.offset_start == b.offset_start
        && a.offset_end == b.offset_end
        && a.stream == b.stream
        && a.size() == b.size()
        && a.service == b.service
}

#[cfg(test)]
mod tests {
    use super::*;
    use cubecl_common::device::{DeviceId, ServiceId};
    use cubecl_environment::stream::StreamId;

    struct FirstService;
    struct SecondService;

    fn handle() -> Handle {
        Handle::new(
            ServiceId::of::<FirstService>(DeviceId::new(1, 0)),
            StreamId { value: 7 },
            4096,
        )
    }

    #[test]
    fn matching_view_is_accepted() {
        let a = handle().offset_start(256).offset_end(512);
        assert!(same_view(&a, &a.clone()));
    }

    #[test]
    fn different_allocation_is_rejected() {
        let a = handle();
        let b = handle();
        assert_ne!(a.memory.descriptor().id, b.memory.descriptor().id);
        assert!(!same_view(&a, &b));
    }

    #[test]
    fn same_device_different_service_type_is_rejected() {
        let a = handle().offset_start(256).offset_end(512);
        let mut b = a.clone();
        b.service = ServiceId::of::<SecondService>(a.service.device);
        // All five original predicates still match. Only the service type
        // differs, so a device-only check would also fail this control.
        assert_eq!(a.memory.descriptor().id, b.memory.descriptor().id);
        assert_eq!(a.offset_start, b.offset_start);
        assert_eq!(a.offset_end, b.offset_end);
        assert_eq!(a.stream, b.stream);
        assert_eq!(a.size(), b.size());
        assert_eq!(a.service.device, b.service.device);
        assert_ne!(a.service.service, b.service.service);
        assert!(same_view(&a, &a.clone()));
        assert!(!same_view(&a, &b));
    }

    #[test]
    fn same_service_type_different_device_is_rejected() {
        let a = handle();
        let mut b = a.clone();
        b.service = ServiceId::of::<FirstService>(DeviceId::new(1, 1));
        assert_eq!(a.service.service, b.service.service);
        assert_ne!(a.service.device, b.service.device);
        assert!(!same_view(&a, &b));
    }

    #[test]
    fn distinct_slice_start_is_rejected() {
        let original = handle();
        let a = original.clone().offset_start(256).offset_end(512);
        let b = original.offset_start(512).offset_end(512);
        assert_eq!(a.memory.descriptor().id, b.memory.descriptor().id);
        assert_eq!(a.offset_end, b.offset_end);
        assert!(!same_view(&a, &b));
    }

    #[test]
    fn distinct_slice_end_is_rejected() {
        let original = handle();
        let a = original.clone().offset_start(256).offset_end(256);
        let b = original.offset_start(256).offset_end(512);
        assert_eq!(a.memory.descriptor().id, b.memory.descriptor().id);
        assert_eq!(a.offset_start, b.offset_start);
        assert!(!same_view(&a, &b));
    }

    #[test]
    fn absent_offsets_are_not_normalized_to_zero() {
        let a = handle();
        assert!(!same_view(&a, &a.clone().offset_start(0)));
        assert!(!same_view(&a, &a.clone().offset_end(0)));
    }

    #[test]
    fn different_stream_is_rejected() {
        let a = handle();
        let mut b = a.clone();
        b.stream = StreamId {
            value: a.stream.value + 1,
        };
        assert!(!same_view(&a, &b));
    }

    #[test]
    fn different_underlying_size_is_rejected() {
        let a = handle();
        let b = Handle::from_memory(a.memory.clone(), a.service, a.stream, a.size() + 1);
        assert_eq!(a.memory.descriptor().id, b.memory.descriptor().id);
        assert_eq!(a.offset_start, b.offset_start);
        assert_eq!(a.offset_end, b.offset_end);
        assert!(!same_view(&a, &b));
    }
}
