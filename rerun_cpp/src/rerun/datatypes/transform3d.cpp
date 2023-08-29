// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/datatypes/transform3d.fbs"

#include "transform3d.hpp"

#include "translation_and_mat3x3.hpp"
#include "translation_rotation_scale3d.hpp"

#include <arrow/builder.h>
#include <arrow/type_fwd.h>

namespace rerun {
    namespace datatypes {
        const std::shared_ptr<arrow::DataType> &Transform3D::arrow_field() {
            static const auto datatype = arrow::dense_union({
                arrow::field("_null_markers", arrow::null(), true, nullptr),
                arrow::field(
                    "TranslationAndMat3x3",
                    rerun::datatypes::TranslationAndMat3x3::arrow_field(),
                    false
                ),
                arrow::field(
                    "TranslationRotationScale",
                    rerun::datatypes::TranslationRotationScale3D::arrow_field(),
                    false
                ),
            });
            return datatype;
        }

        Result<std::shared_ptr<arrow::DenseUnionBuilder>> Transform3D::new_arrow_array_builder(
            arrow::MemoryPool *memory_pool
        ) {
            if (!memory_pool) {
                return Error(ErrorCode::UnexpectedNullArgument, "Memory pool is null.");
            }

            return Result(std::make_shared<arrow::DenseUnionBuilder>(
                memory_pool,
                std::vector<std::shared_ptr<arrow::ArrayBuilder>>({
                    std::make_shared<arrow::NullBuilder>(memory_pool),
                    rerun::datatypes::TranslationAndMat3x3::new_arrow_array_builder(memory_pool)
                        .value,
                    rerun::datatypes::TranslationRotationScale3D::new_arrow_array_builder(
                        memory_pool
                    )
                        .value,
                }),
                arrow_field()
            ));
        }

        Error Transform3D::fill_arrow_array_builder(
            arrow::DenseUnionBuilder *builder, const Transform3D *elements, size_t num_elements
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

            ARROW_RETURN_NOT_OK(builder->Reserve(static_cast<int64_t>(num_elements)));
            for (size_t elem_idx = 0; elem_idx < num_elements; elem_idx += 1) {
                const auto &union_instance = elements[elem_idx];
                ARROW_RETURN_NOT_OK(builder->Append(static_cast<int8_t>(union_instance._tag)));

                auto variant_index = static_cast<int>(union_instance._tag);
                auto variant_builder_untyped = builder->child_builder(variant_index).get();

                switch (union_instance._tag) {
                    case detail::Transform3DTag::NONE: {
                        ARROW_RETURN_NOT_OK(variant_builder_untyped->AppendNull());
                        break;
                    }
                    case detail::Transform3DTag::TranslationAndMat3x3: {
                        auto variant_builder =
                            static_cast<arrow::StructBuilder *>(variant_builder_untyped);
                        RR_RETURN_NOT_OK(
                            rerun::datatypes::TranslationAndMat3x3::fill_arrow_array_builder(
                                variant_builder,
                                &union_instance._data.translation_and_mat3x3,
                                1
                            )
                        );
                        break;
                    }
                    case detail::Transform3DTag::TranslationRotationScale: {
                        auto variant_builder =
                            static_cast<arrow::StructBuilder *>(variant_builder_untyped);
                        RR_RETURN_NOT_OK(
                            rerun::datatypes::TranslationRotationScale3D::fill_arrow_array_builder(
                                variant_builder,
                                &union_instance._data.translation_rotation_scale,
                                1
                            )
                        );
                        break;
                    }
                }
            }

            return Error::ok();
        }
    } // namespace datatypes
} // namespace rerun
