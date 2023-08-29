// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/components/instance_key.fbs"

#pragma once

#include "../data_cell.hpp"
#include "../result.hpp"

#include <cstdint>
#include <memory>
#include <utility>

namespace arrow {
    template <typename T>
    class NumericBuilder;

    class DataType;
    class MemoryPool;
    class UInt64Type;
    using UInt64Builder = NumericBuilder<UInt64Type>;
} // namespace arrow

namespace rerun {
    namespace components {
        /// A unique numeric identifier for each individual instance within a batch.
        struct InstanceKey {
            uint64_t value;

            /// Name of the component, used for serialization.
            static const char* NAME;

          public:
            InstanceKey() = default;

            InstanceKey(uint64_t _value) : value(std::move(_value)) {}

            InstanceKey& operator=(uint64_t _value) {
                value = std::move(_value);
                return *this;
            }

            /// Returns the arrow data type this type corresponds to.
            static const std::shared_ptr<arrow::DataType>& arrow_field();

            /// Creates a new array builder with an array of this type.
            static Result<std::shared_ptr<arrow::UInt64Builder>> new_arrow_array_builder(
                arrow::MemoryPool* memory_pool
            );

            /// Fills an arrow array builder with an array of this type.
            static Error fill_arrow_array_builder(
                arrow::UInt64Builder* builder, const InstanceKey* elements, size_t num_elements
            );

            /// Creates a Rerun DataCell from an array of InstanceKey components.
            static Result<rerun::DataCell> to_data_cell(
                const InstanceKey* instances, size_t num_instances
            );
        };
    } // namespace components
} // namespace rerun
