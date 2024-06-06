from __future__ import annotations

import math
import os
from argparse import Namespace
from typing import Any, Iterable
from uuid import uuid4

import numpy as np
import rerun as rr

README = """
# Component UI

In the streams view, for each top-level entity, select it, and check that the component list in the selection panel looks Nice(tm).
"""


class TestCase:
    """
    Test case information.

    Usage:
    - For component which are typically used in batch (e.g. Point2D), use the `batch` keyword argument.
    - For union component (e.g. Rotation3D), use the `alternatives` keyword argument.
    - Otherwise, use the `single` positional argument.
    - To exclude a component, use `disabled=True`
    """

    def __init__(
        self,
        single: Any | None = None,
        *,
        batch: Iterable[Any] | None = None,
        alternatives: Iterable[Any] | None = None,
        disabled: bool = False,
    ):
        assert (
            (single is not None) ^ (batch is not None) ^ (alternatives is not None) ^ disabled
        ), "Exactly one of single, batch, or alternatives must be provided"

        if batch is not None:
            batch = list(batch)
            assert len(batch) > 1, "Batch must have at least two elements"

        if alternatives is not None:
            alternatives = list(alternatives)
            assert len(alternatives) > 1, "Alternatives must have at least two elements"

        self._single = single
        self._batch = batch
        self._alternatives = alternatives
        self._disabled = disabled

    def disabled(self) -> bool:
        return self._disabled

    def single(self) -> Any:
        if self._single is not None:
            return self._single
        elif self._batch is not None:
            return self._batch[0]
        elif self._alternatives is not None:
            return self._alternatives[0]
        assert False, "Unreachable"

    def batch(self) -> list[Any] | None:
        return self._batch

    def alternatives(self) -> list[Any] | None:
        if self._alternatives is not None:
            return self._alternatives[1:]
        else:
            return None


ALL_COMPONENTS: dict[str, TestCase] = {
    "AnnotationContextBatch": TestCase([
        rr.datatypes.ClassDescriptionMapElem(
            class_id=1,
            class_description=rr.datatypes.ClassDescription(
                info=rr.datatypes.AnnotationInfo(id=1, label="label", color=(255, 0, 0, 255)),
                keypoint_annotations=[
                    rr.datatypes.AnnotationInfo(id=1, label="one", color=(255, 0, 0, 255)),
                    rr.datatypes.AnnotationInfo(id=2, label="two", color=(255, 255, 0, 255)),
                    rr.datatypes.AnnotationInfo(id=3, label="three", color=(255, 0, 255, 255)),
                ],
                keypoint_connections=[(1, 2), (2, 3), (3, 1)],
            ),
        )
    ]),
    "BlobBatch": TestCase(
        alternatives=[
            b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09",
            np.random.randint(0, 255, (10, 10), dtype=np.uint8).tobytes(),
        ]
    ),
    "ClassIdBatch": TestCase(batch=[1, 2, 3, 6]),
    "ClearIsRecursiveBatch": TestCase(disabled=True),  # disabled because it messes with the logging
    "ColorBatch": TestCase(batch=[(255, 0, 0, 255), (0, 255, 0, 255), (0, 0, 255, 255)]),
    "DepthMeterBatch": TestCase(1000.0),
    "DisconnectedSpaceBatch": TestCase(True),
    "DrawOrderBatch": TestCase(100.0),
    "HalfSizes2DBatch": TestCase(batch=[(5.0, 10.0), (50, 30), (23, 45)]),
    "HalfSizes3DBatch": TestCase(batch=[(5.0, 10.0, 20.0), (50, 30, 40), (23, 45, 67)]),
    "ImagePlaneDistanceBatch": TestCase(batch=[100.0, 200.0, 300.0]),
    "KeypointIdBatch": TestCase(batch=[5, 6, 7]),
    "LineStrip2DBatch": TestCase(batch=[((0, 0), (1, 1), (2, 2)), ((3, 3), (4, 4), (5, 5)), ((6, 6), (7, 7), (8, 8))]),
    "LineStrip3DBatch": TestCase(
        batch=[((0, 0, 0), (1, 1, 1), (2, 2, 2)), ((3, 3, 3), (4, 4, 4), (5, 5, 5)), ((6, 6, 6), (7, 7, 7), (8, 8, 8))]
    ),
    "MarkerShapeBatch": TestCase(
        batch=[rr.components.MarkerShape.Plus, rr.components.MarkerShape.Cross, rr.components.MarkerShape.Circle]
    ),
    "MarkerSizeBatch": TestCase(batch=[5.0, 1.0, 2.0]),
    "MaterialBatch": TestCase(rr.datatypes.Material((255, 255, 0, 255))),
    "MediaTypeBatch": TestCase("application/jpg"),
    "NameBatch": TestCase(batch=["Hello World", "Foo Bar", "Baz Qux"]),
    "OutOfTreeTransform3DBatch": TestCase(
        alternatives=[
            rr.datatypes.TranslationRotationScale3D(
                translation=(1, 2, 3), rotation=rr.datatypes.Quaternion(xyzw=[0, 0, 0, 1]), scale=(1, 1, 1)
            ),
            rr.datatypes.TranslationAndMat3x3(translation=(1, 2, 3), mat3x3=[[1, 0, 0], [0, 1, 0], [0, 0, 1]]),
        ]
    ),
    "PinholeProjectionBatch": TestCase([(0, 1, 2), (3, 4, 5), (6, 7, 8)]),
    "Position2DBatch": TestCase(batch=[(0, 1), (2, 3), (4, 5)]),
    "Position3DBatch": TestCase(batch=[(0, 3, 4), (1, 4, 5), (2, 5, 6)]),
    "RadiusBatch": TestCase(batch=[4.5, 5, 6, 7]),
    "Range1DBatch": TestCase((0, 5)),
    "ResolutionBatch": TestCase((1920, 1080)),
    "Rotation3DBatch": TestCase(
        alternatives=[
            rr.datatypes.Quaternion(xyzw=[0, 0, 0, 1]),
            rr.datatypes.RotationAxisAngle(axis=(1, 0, 0), angle=rr.datatypes.Angle(rad=math.pi)),
            rr.datatypes.RotationAxisAngle(axis=(1, 0, 0), angle=rr.datatypes.Angle(deg=180)),
        ]
    ),
    "ScalarBatch": TestCase(3),
    "StrokeWidthBatch": TestCase(2.0),
    "TensorDataBatch": TestCase(
        alternatives=[
            rr.datatypes.TensorData(array=np.random.randint(0, 255, (10, 10), dtype=np.uint8)),
            rr.datatypes.TensorData(array=np.random.randint(0, 255, (10, 10, 3), dtype=np.uint8)),
            rr.datatypes.TensorData(array=np.random.randint(0, 255, (5, 3, 6, 4), dtype=np.uint8)),
            rr.datatypes.TensorData(
                array=np.random.randint(0, 255, (5, 3, 6, 4), dtype=np.uint8),
                dim_names=[None, "hello", None, "world"],
            ),
            rr.datatypes.TensorData(array=np.random.randint(0, 255, (5, 3, 6, 4, 3), dtype=np.uint8)),
        ]
    ),
    "Texcoord2DBatch": TestCase(batch=[(0, 0), (1, 1), (2, 2)]),
    "TextBatch": TestCase("Hello world"),
    "TextLogLevelBatch": TestCase(batch=["INFO", "CRITICAL", "WARNING"]),
    "Transform3DBatch": TestCase(
        alternatives=[
            rr.datatypes.TranslationRotationScale3D(
                translation=(1, 2, 3), rotation=rr.datatypes.Quaternion(xyzw=[0, 0, 0, 1]), scale=(1, 1, 1)
            ),
            rr.datatypes.TranslationAndMat3x3(translation=(1, 2, 3), mat3x3=[[1, 0, 0], [0, 1, 0], [0, 0, 1]]),
        ]
    ),
    "TriangleIndicesBatch": TestCase(batch=[(0, 1, 2), (3, 4, 5), (6, 7, 8)]),
    "Vector2DBatch": TestCase(batch=[(0, 1), (2, 3), (4, 5)]),
    "Vector3DBatch": TestCase(batch=[(0, 3, 4), (1, 4, 5), (2, 5, 6)]),
    "ViewCoordinatesBatch": TestCase(rr.components.ViewCoordinates.LBD),
    "VisualizerOverridesBatch": TestCase(disabled=True),  # no Python-based serialization
}


def log_readme() -> None:
    rr.log("readme", rr.TextDocument(README, media_type=rr.MediaType.MARKDOWN), static=True)


def log_some_space_views() -> None:
    # check that we didn't forget a component
    missing_components = set(c for c in dir(rr.components) if c.endswith("Batch")) - set(ALL_COMPONENTS.keys())
    assert (
        len(missing_components) == 0
    ), f"Some components are missing from the `ALL_COMPONENTS` dictionary: {missing_components}"

    # log all components as len=1 batches
    rr.log_components(
        "all_components_single",
        [
            getattr(rr.components, component_name)(test_case.single())
            for component_name, test_case in ALL_COMPONENTS.items()
            if not test_case.disabled()
        ],
    )

    # log all components as batches (except those for which it doesn't make sense)
    rr.log_components(
        "all_components_batches",
        [
            getattr(rr.components, component_name)(test_case.batch())
            for component_name, test_case in ALL_COMPONENTS.items()
            if test_case.batch() is not None
        ],
    )

    # log all alternative values for components
    for component_name, test_case in ALL_COMPONENTS.items():
        alternatives = test_case.alternatives()
        if alternatives is None:
            continue

        for i, alternative in enumerate(alternatives):
            rr.log_components(
                f"all_components_alternative_{i}",
                [getattr(rr.components, component_name)(alternative)],
            )


def run(args: Namespace) -> None:
    rr.script_setup(args, f"{os.path.basename(__file__)}", recording_id=uuid4())

    log_readme()
    log_some_space_views()


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Interactive release checklist")
    rr.script_add_args(parser)
    args = parser.parse_args()
    run(args)
