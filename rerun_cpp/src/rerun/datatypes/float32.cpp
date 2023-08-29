// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/datatypes/scalars.fbs"

#include "float32.hpp"

#include <arrow/builder.h>
#include <arrow/type_fwd.h>

namespace rerun {
    namespace datatypes {
        const std::shared_ptr<arrow::DataType>& Float32::arrow_field() {
            static const auto datatype = arrow::float32();
            return datatype;
        }

        Result<std::shared_ptr<arrow::FloatBuilder>> Float32::new_arrow_array_builder(
            arrow::MemoryPool* memory_pool
        ) {
            if (!memory_pool) {
                return Error(ErrorCode::UnexpectedNullArgument, "Memory pool is null.");
            }

            return Result(std::make_shared<arrow::FloatBuilder>(memory_pool));
        }

        Error Float32::fill_arrow_array_builder(
            arrow::FloatBuilder* builder, const Float32* elements, size_t num_elements
        ) {
            if (!builder) {
                return Error(ErrorCode::UnexpectedNullArgument, "Passed array builder is null.");
            }
            if (!elements) {
                return Error(
                    ErrorCode::UnexpectedNullArgument,
                    "Cannot serialize null pointer to arrow array."
                );
            }

            static_assert(sizeof(*elements) == sizeof(elements->value));
            ARROW_RETURN_NOT_OK(
                builder->AppendValues(&elements->value, static_cast<int64_t>(num_elements))
            );

            return Error::ok();
        }
    } // namespace datatypes
} // namespace rerun
