// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.

#![allow(trivial_numeric_casts)]
#![allow(unused_parens)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::iter_on_single_items)]
#![allow(clippy::map_flatten)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::needless_question_mark)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_cast)]

/// A 3x3 column-major Matrix.
#[derive(Clone, Debug, Default, Copy, PartialEq, PartialOrd)]
pub struct Mat3x3(pub [f32; 9usize]);

impl<'a> From<Mat3x3> for ::std::borrow::Cow<'a, Mat3x3> {
    #[inline]
    fn from(value: Mat3x3) -> Self {
        std::borrow::Cow::Owned(value)
    }
}

impl<'a> From<&'a Mat3x3> for ::std::borrow::Cow<'a, Mat3x3> {
    #[inline]
    fn from(value: &'a Mat3x3) -> Self {
        std::borrow::Cow::Borrowed(value)
    }
}

impl crate::Loggable for Mat3x3 {
    type Name = crate::DatatypeName;

    #[inline]
    fn name() -> Self::Name {
        "rerun.datatypes.Mat3x3".into()
    }

    #[allow(unused_imports, clippy::wildcard_imports)]
    #[inline]
    fn arrow_datatype() -> arrow2::datatypes::DataType {
        use ::arrow2::datatypes::*;
        DataType::FixedSizeList(
            Box::new(Field {
                name: "item".to_owned(),
                data_type: DataType::Float32,
                is_nullable: false,
                metadata: [].into(),
            }),
            9usize,
        )
    }

    #[allow(unused_imports, clippy::wildcard_imports)]
    fn try_to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<::std::borrow::Cow<'a, Self>>>>,
        extension_wrapper: Option<&str>,
    ) -> crate::SerializationResult<Box<dyn ::arrow2::array::Array>>
    where
        Self: Clone + 'a,
    {
        use crate::{Loggable as _, ResultExt as _};
        use ::arrow2::{array::*, datatypes::*};
        _ = extension_wrapper;
        Ok({
            let (somes, data0): (Vec<_>, Vec<_>) = data
                .into_iter()
                .map(|datum| {
                    let datum: Option<::std::borrow::Cow<'a, Self>> = datum.map(Into::into);
                    let datum = datum.map(|datum| {
                        let Self(data0) = datum.into_owned();
                        data0
                    });
                    (datum.is_some(), datum)
                })
                .unzip();
            let data0_bitmap: Option<::arrow2::bitmap::Bitmap> = {
                let any_nones = somes.iter().any(|some| !*some);
                any_nones.then(|| somes.into())
            };
            {
                use arrow2::{buffer::Buffer, offset::OffsetsBuffer};
                let data0_inner_data: Vec<_> = data0
                    .iter()
                    .flatten()
                    .flatten()
                    .cloned()
                    .map(Some)
                    .collect();
                let data0_inner_bitmap: Option<::arrow2::bitmap::Bitmap> =
                    data0_bitmap.as_ref().map(|bitmap| {
                        bitmap
                            .iter()
                            .map(|i| std::iter::repeat(i).take(9usize))
                            .flatten()
                            .collect::<Vec<_>>()
                            .into()
                    });
                FixedSizeListArray::new(
                    Self::arrow_datatype(),
                    PrimitiveArray::new(
                        DataType::Float32,
                        data0_inner_data
                            .into_iter()
                            .map(|v| v.unwrap_or_default())
                            .collect(),
                        data0_inner_bitmap,
                    )
                    .boxed(),
                    data0_bitmap,
                )
                .boxed()
            }
        })
    }

    #[allow(unused_imports, clippy::wildcard_imports)]
    fn try_from_arrow_opt(
        data: &dyn ::arrow2::array::Array,
    ) -> crate::DeserializationResult<Vec<Option<Self>>>
    where
        Self: Sized,
    {
        use crate::{Loggable as _, ResultExt as _};
        use ::arrow2::{array::*, buffer::*, datatypes::*};
        Ok({
            let data = data
                .as_any()
                .downcast_ref::<::arrow2::array::FixedSizeListArray>()
                .ok_or_else(|| {
                    crate::DeserializationError::datatype_mismatch(
                        DataType::FixedSizeList(
                            Box::new(Field {
                                name: "item".to_owned(),
                                data_type: DataType::Float32,
                                is_nullable: false,
                                metadata: [].into(),
                            }),
                            9usize,
                        ),
                        data.data_type().clone(),
                    )
                })
                .with_context("rerun.datatypes.Mat3x3#coeffs")?;
            if data.is_empty() {
                Vec::new()
            } else {
                let offsets = (0..)
                    .step_by(9usize)
                    .zip((9usize..).step_by(9usize).take(data.len()));
                let data_inner = {
                    let data_inner = &**data.values();
                    data_inner
                        .as_any()
                        .downcast_ref::<Float32Array>()
                        .ok_or_else(|| {
                            crate::DeserializationError::datatype_mismatch(
                                DataType::Float32,
                                data_inner.data_type().clone(),
                            )
                        })
                        .with_context("rerun.datatypes.Mat3x3#coeffs")?
                        .into_iter()
                        .map(|opt| opt.copied())
                        .collect::<Vec<_>>()
                };
                arrow2::bitmap::utils::ZipValidity::new_with_validity(offsets, data.validity())
                    .map(|elem| {
                        elem.map(|(start, end)| {
                            debug_assert!(end - start == 9usize);
                            if end as usize > data_inner.len() {
                                return Err(crate::DeserializationError::offset_slice_oob(
                                    (start, end),
                                    data_inner.len(),
                                ));
                            }

                            #[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
                            let data =
                                unsafe { data_inner.get_unchecked(start as usize..end as usize) };
                            let data = data.iter().cloned().map(Option::unwrap_or_default);
                            let arr = array_init::from_iter(data).unwrap();
                            Ok(arr)
                        })
                        .transpose()
                    })
                    .collect::<crate::DeserializationResult<Vec<Option<_>>>>()?
            }
            .into_iter()
        }
        .map(|v| v.ok_or_else(crate::DeserializationError::missing_data))
        .map(|res| res.map(|v| Some(Self(v))))
        .collect::<crate::DeserializationResult<Vec<Option<_>>>>()
        .with_context("rerun.datatypes.Mat3x3#coeffs")
        .with_context("rerun.datatypes.Mat3x3")?)
    }
}
