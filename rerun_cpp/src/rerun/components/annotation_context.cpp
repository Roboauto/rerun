// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/components/annotation_context.fbs"

#include "annotation_context.hpp"

#include "../arrow.hpp"
#include "../datatypes/class_description_map_elem.hpp"

#include <arrow/builder.h>
#include <arrow/table.h>
#include <arrow/type_fwd.h>

namespace rerun {
    namespace components {
        const char *AnnotationContext::NAME = "rerun.annotation_context";

        const std::shared_ptr<arrow::DataType> &AnnotationContext::arrow_field() {
            static const auto datatype = arrow::list(arrow::field(
                "item",
                rerun::datatypes::ClassDescriptionMapElem::arrow_field(),
                false
            ));
            return datatype;
        }

        Result<std::shared_ptr<arrow::ListBuilder>> AnnotationContext::new_arrow_array_builder(
            arrow::MemoryPool *memory_pool
        ) {
            if (!memory_pool) {
                return Error(ErrorCode::UnexpectedNullArgument, "Memory pool is null.");
            }

            return Result(std::make_shared<arrow::ListBuilder>(
                memory_pool,
                rerun::datatypes::ClassDescriptionMapElem::new_arrow_array_builder(memory_pool)
                    .value
            ));
        }

        Error AnnotationContext::fill_arrow_array_builder(
            arrow::ListBuilder *builder, const AnnotationContext *elements, size_t num_elements
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

            auto value_builder = static_cast<arrow::StructBuilder *>(builder->value_builder());
            ARROW_RETURN_NOT_OK(builder->Reserve(static_cast<int64_t>(num_elements)));
            ARROW_RETURN_NOT_OK(value_builder->Reserve(static_cast<int64_t>(num_elements * 2)));

            for (size_t elem_idx = 0; elem_idx < num_elements; elem_idx += 1) {
                const auto &element = elements[elem_idx];
                ARROW_RETURN_NOT_OK(builder->Append());
                if (element.class_map.data()) {
                    RR_RETURN_NOT_OK(
                        rerun::datatypes::ClassDescriptionMapElem::fill_arrow_array_builder(
                            value_builder,
                            element.class_map.data(),
                            element.class_map.size()
                        )
                    );
                }
            }

            return Error::ok();
        }

        Result<rerun::DataCell> AnnotationContext::to_data_cell(
            const AnnotationContext *instances, size_t num_instances
        ) {
            // TODO(andreas): Allow configuring the memory pool.
            arrow::MemoryPool *pool = arrow::default_memory_pool();

            auto builder_result = AnnotationContext::new_arrow_array_builder(pool);
            RR_RETURN_NOT_OK(builder_result.error);
            auto builder = std::move(builder_result.value);
            if (instances && num_instances > 0) {
                RR_RETURN_NOT_OK(AnnotationContext::fill_arrow_array_builder(
                    builder.get(),
                    instances,
                    num_instances
                ));
            }
            std::shared_ptr<arrow::Array> array;
            ARROW_RETURN_NOT_OK(builder->Finish(&array));

            auto schema = arrow::schema(
                {arrow::field(AnnotationContext::NAME, AnnotationContext::arrow_field(), false)}
            );

            rerun::DataCell cell;
            cell.component_name = AnnotationContext::NAME;
            const auto ipc_result = rerun::ipc_from_table(*arrow::Table::Make(schema, {array}));
            RR_RETURN_NOT_OK(ipc_result.error);
            cell.buffer = std::move(ipc_result.value);

            return cell;
        }
    } // namespace components
} // namespace rerun
