// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/testing/components/fuzzy.fbs"

#pragma once

#include <cstdint>
#include <optional>
#include <utility>

#include "../datatypes/affix_fuzzer1.hpp"

namespace rr {
    namespace components {
        struct AffixFuzzer4 {
            std::optional<rr::datatypes::AffixFuzzer1> single_optional;

            AffixFuzzer4(std::optional<rr::datatypes::AffixFuzzer1> single_optional)
                : single_optional(std::move(single_optional)) {}
        };
    } // namespace components
} // namespace rr
