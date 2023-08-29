// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/datatypes/color.fbs"

#pragma once

#include "../result.hpp"

#include <cstdint>
#include <memory>
#include <utility>

namespace arrow {
    template <typename T>
    class NumericBuilder;

    class DataType;
    class MemoryPool;
    class UInt32Type;
    using UInt32Builder = NumericBuilder<UInt32Type>;
} // namespace arrow

namespace rerun {
    namespace datatypes {
        /// An RGBA color tuple with unmultiplied/separate alpha, in sRGB gamma space with linear
        /// alpha.
        struct Color {
            uint32_t rgba;

          public:
            // Extensions to generated type defined in 'color_ext.cpp'

            /// Construct Color from unmultiplied RGBA values.
            Color(uint8_t r, uint8_t g, uint8_t b, uint8_t a = 255)
                : Color(static_cast<uint32_t>((r << 24) | (g << 16) | (b << 8) | a)) {}

            /// Construct Color from unmultiplied RGBA values.
            Color(const uint8_t (&_rgba)[4]) : Color(_rgba[0], _rgba[1], _rgba[2], _rgba[3]) {}

            /// Construct Color from RGB values, setting alpha to 255.
            Color(const uint8_t (&_rgb)[3]) : Color(_rgb[0], _rgb[1], _rgb[2]) {}

            uint8_t r() const {
                return static_cast<uint8_t>((rgba >> 24) & 0xFF);
            }

            uint8_t g() const {
                return static_cast<uint8_t>((rgba >> 16) & 0xFF);
            }

            uint8_t b() const {
                return static_cast<uint8_t>((rgba >> 8) & 0xFF);
            }

            uint8_t a() const {
                return static_cast<uint8_t>(rgba & 0xFF);
            }

          public:
            Color() = default;

            Color(uint32_t _rgba) : rgba(std::move(_rgba)) {}

            Color& operator=(uint32_t _rgba) {
                rgba = std::move(_rgba);
                return *this;
            }

            /// Returns the arrow data type this type corresponds to.
            static const std::shared_ptr<arrow::DataType>& arrow_field();

            /// Creates a new array builder with an array of this type.
            static Result<std::shared_ptr<arrow::UInt32Builder>> new_arrow_array_builder(
                arrow::MemoryPool* memory_pool
            );

            /// Fills an arrow array builder with an array of this type.
            static Error fill_arrow_array_builder(
                arrow::UInt32Builder* builder, const Color* elements, size_t num_elements
            );
        };
    } // namespace datatypes
} // namespace rerun
