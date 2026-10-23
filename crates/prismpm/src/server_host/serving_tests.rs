//! HO-24: the generated listener and its RFC 9112 framing (R3 skeleton).

macro_rules! skeleton {
    ($($name:ident),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                panic!("R3 skeleton: the serving effect is not generated yet");
            }
        )*
    };
}

skeleton!(
    the_listener_binds_loopback_only_and_refuses_any_other_address,
    a_malformed_request_line_is_a_400_before_the_core_is_called,
    an_oversized_header_section_is_a_431,
    too_many_header_fields_are_a_431,
    an_oversized_body_is_a_413_by_content_length_and_by_chunks,
    a_forbidden_origin_is_a_403_before_the_core_is_called,
    accept_and_content_type_are_required_from_the_model,
    fields_reach_the_core_in_received_order_without_merging,
    content_length_and_chunked_bodies_are_delimited_by_rfc_9112,
    host_provided_frame_sections_come_from_declared_effects,
    the_core_response_is_serialized_as_returned,
    every_served_vector_is_answered_over_a_real_socket,
);
