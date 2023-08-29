// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/testing/components/fuzzy.fbs"

#pragma once

#include "../datatypes/affix_fuzzer4.hpp"
#include "../datatypes/affix_fuzzer5.hpp"

#include <cstdint>
#include <memory>
#include <optional>
#include <rerun/data_cell.hpp>
#include <rerun/result.hpp>
#include <utility>

namespace arrow {
    class DataType;
    class MemoryPool;
    class StructBuilder;
} // namespace arrow

namespace rerun {
    namespace components {
        struct AffixFuzzer19 {
            rerun::datatypes::AffixFuzzer5 just_a_table_nothing_shady;

            /// Name of the component, used for serialization.
            static const char* NAME;

          public:
            AffixFuzzer19() = default;

            AffixFuzzer19(rerun::datatypes::AffixFuzzer5 _just_a_table_nothing_shady)
                : just_a_table_nothing_shady(std::move(_just_a_table_nothing_shady)) {}

            AffixFuzzer19& operator=(rerun::datatypes::AffixFuzzer5 _just_a_table_nothing_shady) {
                just_a_table_nothing_shady = std::move(_just_a_table_nothing_shady);
                return *this;
            }

            AffixFuzzer19(std::optional<rerun::datatypes::AffixFuzzer4> arg)
                : just_a_table_nothing_shady(std::move(arg)) {}

            /// Returns the arrow data type this type corresponds to.
            static const std::shared_ptr<arrow::DataType>& arrow_field();

            /// Creates a new array builder with an array of this type.
            static Result<std::shared_ptr<arrow::StructBuilder>> new_arrow_array_builder(
                arrow::MemoryPool* memory_pool
            );

            /// Fills an arrow array builder with an array of this type.
            static Error fill_arrow_array_builder(
                arrow::StructBuilder* builder, const AffixFuzzer19* elements, size_t num_elements
            );

            /// Creates a Rerun DataCell from an array of AffixFuzzer19 components.
            static Result<rerun::DataCell> to_data_cell(
                const AffixFuzzer19* instances, size_t num_instances
            );
        };
    } // namespace components
} // namespace rerun
