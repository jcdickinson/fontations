//! the [GPOS] table
//!
//! [GPOS]: https://docs.microsoft.com/en-us/typography/opentype/spec/gpos

#[path = "./value_record.rs"]
mod value_record;

use crate::array::ComputedArray;

/// reexport stuff from layout that we use
pub use super::layout::{
    ClassDef, CoverageTable, Device, DeviceOrVariationIndex, FeatureList, FeatureVariations,
    Lookup, ScriptList,
};
pub use value_record::ValueRecord;

#[cfg(test)]
#[path = "../tests/test_gpos.rs"]
mod spec_tests;

include!("../../generated/generated_gpos.rs");

/// A typed GPOS [LookupList](super::layout::LookupList) table
pub type PositionLookupList<'a> = super::layout::LookupList<'a, PositionLookup<'a>>;

/// A GPOS [SequenceContext](super::layout::SequenceContext)
pub type PositionSequenceContext<'a> = super::layout::SequenceContext<'a>;

/// A GPOS [ChainedSequenceContext](super::layout::ChainedSequenceContext)
pub type PositionChainContext<'a> = super::layout::ChainedSequenceContext<'a>;

impl<'a> AnchorTable<'a> {
    /// Attempt to resolve the `Device` or `VariationIndex` table for the
    /// x_coordinate, if present
    pub fn x_device(&self) -> Option<Result<DeviceOrVariationIndex<'a>, ReadError>> {
        match self {
            AnchorTable::Format3(inner) => inner.x_device(),
            _ => None,
        }
    }

    /// Attempt to resolve the `Device` or `VariationIndex` table for the
    /// y_coordinate, if present
    pub fn y_device(&self) -> Option<Result<DeviceOrVariationIndex<'a>, ReadError>> {
        match self {
            AnchorTable::Format3(inner) => inner.y_device(),
            _ => None,
        }
    }
}
