// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/store/re_types/definitions/rerun/archetypes/disconnected_space.fbs".

#pragma once

#include "../collection.hpp"
#include "../component_batch.hpp"
#include "../components/disconnected_space.hpp"
#include "../indicator_component.hpp"
#include "../result.hpp"

#include <cstdint>
#include <utility>
#include <vector>

namespace rerun::archetypes {
    /// **Archetype**: Spatially disconnect this entity from its parent.
    ///
    /// Specifies that the entity path at which this is logged is spatially disconnected from its parent,
    /// making it impossible to transform the entity path into its parent's space and vice versa.
    /// It *only* applies to space views that work with spatial transformations, i.e. 2D & 3D space views.
    /// This is useful for specifying that a subgraph is independent of the rest of the scene.
    ///
    /// ## Example
    ///
    /// ### Disconnected space
    /// ![image](https://static.rerun.io/disconnected_space/709041fc304b50c74db773b780e32294fe90c95f/full.png)
    ///
    /// ```cpp
    /// #include <rerun.hpp>
    ///
    /// int main() {
    ///     const auto rec = rerun::RecordingStream("rerun_example_disconnected_space");
    ///     rec.spawn().exit_on_failure();
    ///
    ///     // These two points can be projected into the same space..
    ///     rec.log("world/room1/point", rerun::Points3D({{0.0f, 0.0f, 0.0f}}));
    ///     rec.log("world/room2/point", rerun::Points3D({{1.0f, 1.0f, 1.0f}}));
    ///
    ///     // ..but this one lives in a completely separate space!
    ///     rec.log("world/wormhole", rerun::DisconnectedSpace(true));
    ///     rec.log("world/wormhole/point", rerun::Points3D({{2.0f, 2.0f, 2.0f}}));
    /// }
    /// ```
    struct DisconnectedSpace {
        /// Whether the entity path at which this is logged is disconnected from its parent.
        rerun::components::DisconnectedSpace disconnected_space;

      public:
        static constexpr const char IndicatorComponentName[] =
            "rerun.components.DisconnectedSpaceIndicator";

        /// Indicator component, used to identify the archetype when converting to a list of components.
        using IndicatorComponent = rerun::components::IndicatorComponent<IndicatorComponentName>;

      public:
        DisconnectedSpace() = default;
        DisconnectedSpace(DisconnectedSpace&& other) = default;

        explicit DisconnectedSpace(rerun::components::DisconnectedSpace _disconnected_space)
            : disconnected_space(std::move(_disconnected_space)) {}
    };

} // namespace rerun::archetypes

namespace rerun {
    /// \private
    template <typename T>
    struct AsComponents;

    /// \private
    template <>
    struct AsComponents<archetypes::DisconnectedSpace> {
        /// Serialize all set component batches.
        static Result<std::vector<ComponentBatch>> serialize(
            const archetypes::DisconnectedSpace& archetype
        );
    };
} // namespace rerun
