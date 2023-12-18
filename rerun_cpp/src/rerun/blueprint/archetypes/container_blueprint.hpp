// DO NOT EDIT! This file was auto-generated by crates/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/re_types/definitions/rerun/blueprint/archetypes/container_blueprint.fbs".

#pragma once

#include "../../blueprint/components/container_kind.hpp"
#include "../../blueprint/components/included_contents.hpp"
#include "../../blueprint/components/name.hpp"
#include "../../blueprint/components/primary_weights.hpp"
#include "../../blueprint/components/secondary_weights.hpp"
#include "../../blueprint/components/tile_id.hpp"
#include "../../collection.hpp"
#include "../../compiler_utils.hpp"
#include "../../data_cell.hpp"
#include "../../indicator_component.hpp"
#include "../../result.hpp"

#include <cstdint>
#include <optional>
#include <utility>
#include <vector>

namespace rerun::blueprint::archetypes {
    /// **Archetype**: The top-level description of the Viewport.
    struct ContainerBlueprint {
        /// The class of the view.
        rerun::blueprint::components::ContainerKind container_kind;

        /// The name of the container.
        std::optional<rerun::blueprint::components::Name> display_name;

        /// `ContainerIds`s or `SpaceViewId`s that are children of this container.
        std::optional<rerun::blueprint::components::IncludedContents> contents;

        /// The weights of the primary axis. For `Grid` this is the column weights.
        ///
        /// For `Horizontal`/`Vertical` containers, the length of this list should always match the number of contents.
        std::optional<rerun::blueprint::components::PrimaryWeights> primary_weights;

        /// The weights of the secondary axis. For `Grid` this is the row weights. Ignored for `Horizontal`/`Vertical` containers.
        std::optional<rerun::blueprint::components::SecondaryWeights> secondary_weights;

        /// The egui TileId. This is an opaque type so we need to store it via serde.
        std::optional<rerun::blueprint::components::TileId> tile_id;

      public:
        static constexpr const char IndicatorComponentName[] =
            "rerun.blueprint.components.ContainerBlueprintIndicator";

        /// Indicator component, used to identify the archetype when converting to a list of components.
        using IndicatorComponent = rerun::components::IndicatorComponent<IndicatorComponentName>;

      public:
        ContainerBlueprint() = default;
        ContainerBlueprint(ContainerBlueprint&& other) = default;

        explicit ContainerBlueprint(rerun::blueprint::components::ContainerKind _container_kind)
            : container_kind(std::move(_container_kind)) {}

        /// The name of the container.
        ContainerBlueprint with_display_name(rerun::blueprint::components::Name _display_name) && {
            display_name = std::move(_display_name);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// `ContainerIds`s or `SpaceViewId`s that are children of this container.
        ContainerBlueprint with_contents(rerun::blueprint::components::IncludedContents _contents
        ) && {
            contents = std::move(_contents);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// The weights of the primary axis. For `Grid` this is the column weights.
        ///
        /// For `Horizontal`/`Vertical` containers, the length of this list should always match the number of contents.
        ContainerBlueprint with_primary_weights(
            rerun::blueprint::components::PrimaryWeights _primary_weights
        ) && {
            primary_weights = std::move(_primary_weights);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// The weights of the secondary axis. For `Grid` this is the row weights. Ignored for `Horizontal`/`Vertical` containers.
        ContainerBlueprint with_secondary_weights(
            rerun::blueprint::components::SecondaryWeights _secondary_weights
        ) && {
            secondary_weights = std::move(_secondary_weights);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// The egui TileId. This is an opaque type so we need to store it via serde.
        ContainerBlueprint with_tile_id(rerun::blueprint::components::TileId _tile_id) && {
            tile_id = std::move(_tile_id);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// Returns the number of primary instances of this archetype.
        size_t num_instances() const {
            return 1;
        }
    };

} // namespace rerun::blueprint::archetypes

namespace rerun {
    /// \private
    template <typename T>
    struct AsComponents;

    /// \private
    template <>
    struct AsComponents<blueprint::archetypes::ContainerBlueprint> {
        /// Serialize all set component batches.
        static Result<std::vector<DataCell>> serialize(
            const blueprint::archetypes::ContainerBlueprint& archetype
        );
    };
} // namespace rerun
