// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/testing/components/fuzzy.fbs"

#pragma once

#include <cstdint>
#include <utility>

#include "../datatypes/affix_fuzzer5.hpp"

namespace rr {
    namespace components {
        struct AffixFuzzer19 {
            rr::datatypes::AffixFuzzer5 just_a_table_nothing_shady;

            AffixFuzzer19(rr::datatypes::AffixFuzzer5 just_a_table_nothing_shady)
                : just_a_table_nothing_shady(std::move(just_a_table_nothing_shady)) {}
        };
    } // namespace components
} // namespace rr
