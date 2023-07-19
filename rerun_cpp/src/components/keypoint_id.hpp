// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/components/keypoint_id.fbs"

#pragma once

#include <cstdint>
#include <utility>

namespace rr {
    namespace components {
        /// A 16-bit ID representing a type of semantic keypoint within a class.
        struct KeypointId {
            uint16_t id;

            KeypointId(uint16_t id) : id(std::move(id)) {}
        };
    } // namespace components
} // namespace rr
