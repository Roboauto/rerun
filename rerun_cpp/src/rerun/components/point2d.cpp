// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/components/point2d.fbs"

#include "point2d.hpp"

#include "../arrow.hpp"
#include "../datatypes/vec2d.hpp"

#include <arrow/builder.h>
#include <arrow/table.h>
#include <arrow/type_fwd.h>

namespace rerun {
    namespace components {
        const char *Point2D::NAME = "rerun.point2d";

        const std::shared_ptr<arrow::DataType> &Point2D::arrow_field() {
            static const auto datatype = rerun::datatypes::Vec2D::arrow_field();
            return datatype;
        }

        Result<std::shared_ptr<arrow::FixedSizeListBuilder>> Point2D::new_arrow_array_builder(
            arrow::MemoryPool *memory_pool
        ) {
            if (!memory_pool) {
                return Error(ErrorCode::UnexpectedNullArgument, "Memory pool is null.");
            }

            return Result(rerun::datatypes::Vec2D::new_arrow_array_builder(memory_pool).value);
        }

        Error Point2D::fill_arrow_array_builder(
            arrow::FixedSizeListBuilder *builder, const Point2D *elements, size_t num_elements
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

            static_assert(sizeof(rerun::datatypes::Vec2D) == sizeof(Point2D));
            RR_RETURN_NOT_OK(rerun::datatypes::Vec2D::fill_arrow_array_builder(
                builder,
                reinterpret_cast<const rerun::datatypes::Vec2D *>(elements),
                num_elements
            ));

            return Error::ok();
        }

        Result<rerun::DataCell> Point2D::to_data_cell(
            const Point2D *instances, size_t num_instances
        ) {
            // TODO(andreas): Allow configuring the memory pool.
            arrow::MemoryPool *pool = arrow::default_memory_pool();

            auto builder_result = Point2D::new_arrow_array_builder(pool);
            RR_RETURN_NOT_OK(builder_result.error);
            auto builder = std::move(builder_result.value);
            if (instances && num_instances > 0) {
                RR_RETURN_NOT_OK(
                    Point2D::fill_arrow_array_builder(builder.get(), instances, num_instances)
                );
            }
            std::shared_ptr<arrow::Array> array;
            ARROW_RETURN_NOT_OK(builder->Finish(&array));

            auto schema =
                arrow::schema({arrow::field(Point2D::NAME, Point2D::arrow_field(), false)});

            rerun::DataCell cell;
            cell.component_name = Point2D::NAME;
            const auto ipc_result = rerun::ipc_from_table(*arrow::Table::Make(schema, {array}));
            RR_RETURN_NOT_OK(ipc_result.error);
            cell.buffer = std::move(ipc_result.value);

            return cell;
        }
    } // namespace components
} // namespace rerun
