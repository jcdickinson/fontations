//! OpenType font variations common tables.

include!("../../generated/generated_variations.rs");

use super::gvar::SharedTuples;

use std::iter::Skip;

/// Outer and inner indices for reading from an [ItemVariationStore].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeltaSetIndex {
    /// Outer delta set index.
    pub outer: u16,
    /// Inner delta set index.
    pub inner: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleIndex(u16);

impl TupleIndex {
    /// Flag indicating that this tuple variation header includes an embedded
    /// peak tuple record, immediately after the tupleIndex field.
    ///
    /// If set, the low 12 bits of the tupleIndex value are ignored.
    ///
    /// Note that this must always be set within the 'cvar' table.
    pub const EMBEDDED_PEAK_TUPLE: u16 = 0x8000;

    /// Flag indicating that this tuple variation table applies to an
    /// intermediate region within the variation space.
    ///
    /// If set, the header includes the two intermediate-region, start and end
    /// tuple records, immediately after the peak tuple record (if present).
    pub const INTERMEDIATE_REGION: u16 = 0x4000;
    /// Flag indicating that the serialized data for this tuple variation table
    /// includes packed “point” number data.
    ///
    /// If set, this tuple variation table uses that number data; if clear,
    /// this tuple variation table uses shared number data found at the start
    /// of the serialized data for this glyph variation data or 'cvar' table.
    pub const PRIVATE_POINT_NUMBERS: u16 = 0x2000;
    //0x1000	Reserved	Reserved for future use — set to 0.
    //
    /// Mask for the low 12 bits to give the shared tuple records index.
    pub const TUPLE_INDEX_MASK: u16 = 0x0FFF;

    fn tuple_len(self, axis_count: u16, flag: usize) -> usize {
        match flag {
            0 => self.embedded_peak_tuple(),
            1 => self.intermediate_region(),
            _ => panic!("only 0 or 1 allowed here"),
        }
        .then_some(axis_count as usize)
        .unwrap_or_default()
    }

    pub fn bits(self) -> u16 {
        self.0
    }

    pub fn from_bits(bits: u16) -> Self {
        TupleIndex(bits)
    }

    /// `true` if the header includes an embedded peak tuple.
    pub fn embedded_peak_tuple(self) -> bool {
        (self.0 & Self::EMBEDDED_PEAK_TUPLE) != 0
    }

    /// `true` if the header includes the two intermediate region tuple records.
    pub fn intermediate_region(self) -> bool {
        (self.0 & Self::INTERMEDIATE_REGION) != 0
    }

    /// `true` if the data for this table includes packed point number data.
    pub fn private_point_numbers(self) -> bool {
        (self.0 & Self::PRIVATE_POINT_NUMBERS) != 0
    }

    pub fn tuple_records_index(self) -> Option<u16> {
        (!self.embedded_peak_tuple()).then_some(self.0 & Self::TUPLE_INDEX_MASK)
    }
}

impl types::Scalar for TupleIndex {
    type Raw = <u16 as types::Scalar>::Raw;
    fn to_raw(self) -> Self::Raw {
        self.0.to_raw()
    }
    fn from_raw(raw: Self::Raw) -> Self {
        let t = <u16>::from_raw(raw);
        Self(t)
    }
}

/// The 'tupleVariationCount' field of the [Tuple Variation Store Header][header]
///
/// The high 4 bits are flags, and the low 12 bits are the number of tuple
/// variation tables for this glyph. The count can be any number between 1 and 4095.
///
/// [header]: https://learn.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats#tuple-variation-store-header
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleVariationCount(u16);

impl TupleVariationCount {
    /// Flag indicating that some or all tuple variation tables reference a
    /// shared set of “point” numbers.
    ///
    /// These shared numbers are represented as packed point number data at the
    /// start of the serialized data.
    pub const SHARED_POINT_NUMBERS: u16 = 0x8000;

    /// Mask for the low 12 bits to give the shared tuple records index.
    pub const COUNT_MASK: u16 = 0x0FFF;

    pub fn bits(self) -> u16 {
        self.0
    }

    pub fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    /// `true` if any tables reference a shared set of point numbers
    pub fn shared_point_numbers(self) -> bool {
        (self.0 & Self::SHARED_POINT_NUMBERS) != 0
    }

    pub fn count(self) -> u16 {
        self.0 & Self::COUNT_MASK
    }
}

impl types::Scalar for TupleVariationCount {
    type Raw = <u16 as types::Scalar>::Raw;
    fn to_raw(self) -> Self::Raw {
        self.0.to_raw()
    }
    fn from_raw(raw: Self::Raw) -> Self {
        let t = <u16>::from_raw(raw);
        Self(t)
    }
}

impl<'a> TupleVariationHeader<'a> {
    #[cfg(feature = "traversal")]
    fn traverse_tuple_index(&self) -> traversal::FieldType<'a> {
        self.tuple_index().0.into()
    }

    /// Peak tuple record for this tuple variation table — optional,
    /// determined by flags in the tupleIndex value.  Note that this
    /// must always be included in the 'cvar' table.
    pub fn peak_tuple(&self) -> Option<Tuple<'a>> {
        self.tuple_index().embedded_peak_tuple().then(|| {
            let range = self.shape.peak_tuple_byte_range();
            Tuple {
                values: self.data.read_array(range).unwrap(),
            }
        })
    }

    /// Intermediate start tuple record for this tuple variation table
    /// — optional, determined by flags in the tupleIndex value.
    pub fn intermediate_start_tuple(&self) -> Option<Tuple<'a>> {
        self.tuple_index().intermediate_region().then(|| {
            let range = self.shape.intermediate_start_tuple_byte_range();
            Tuple {
                values: self.data.read_array(range).unwrap(),
            }
        })
    }

    /// Intermediate end tuple record for this tuple variation table
    /// — optional, determined by flags in the tupleIndex value.
    pub fn intermediate_end_tuple(&self) -> Option<Tuple<'a>> {
        self.tuple_index().intermediate_region().then(|| {
            let range = self.shape.intermediate_end_tuple_byte_range();
            Tuple {
                values: self.data.read_array(range).unwrap(),
            }
        })
    }

    /// Compute the actual length of this table in bytes
    fn byte_len(&self, axis_count: u16) -> usize {
        const FIXED_LEN: usize = u16::RAW_BYTE_LEN + TupleIndex::RAW_BYTE_LEN;
        let tuple_byte_len = F2Dot14::RAW_BYTE_LEN * axis_count as usize;
        let index = self.tuple_index();
        FIXED_LEN
            + index
                .embedded_peak_tuple()
                .then_some(tuple_byte_len)
                .unwrap_or_default()
            + index
                .intermediate_region()
                .then_some(tuple_byte_len * 2)
                .unwrap_or_default()
    }
}

impl<'a> Tuple<'a> {
    pub fn len(&self) -> usize {
        self.values().len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, idx: usize) -> Option<F2Dot14> {
        self.values.get(idx).map(BigEndian::get)
    }
}

//FIXME: add an #[extra_traits(..)] attribute!
#[allow(clippy::derivable_impls)]
impl Default for Tuple<'_> {
    fn default() -> Self {
        Self {
            values: Default::default(),
        }
    }
}

/// [Packed "Point" Numbers](https://learn.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats#packed-point-numbers)
#[derive(Clone, Debug)]
pub struct PackedPointNumbers<'a> {
    data: FontData<'a>,
}

impl<'a> PackedPointNumbers<'a> {
    /// read point numbers off the front of this data, returning the remaining data
    pub fn split_off_front(data: FontData<'a>) -> (Self, FontData<'a>) {
        let this = PackedPointNumbers { data };
        let total_len = this.total_len();
        let remainder = data.split_off(total_len).unwrap_or_default();
        (this, remainder)
    }

    /// The number of points in this set
    pub fn count(&self) -> u16 {
        self.count_and_count_bytes().0
    }

    /// compute the count, and the number of bytes used to store it
    fn count_and_count_bytes(&self) -> (u16, usize) {
        match self.data.read_at::<u8>(0).unwrap_or(0) {
            0 => (0, 1),
            count @ 1..=127 => (count as u16, 1),
            _ => {
                // "If the high bit of the first byte is set, then a second byte is used.
                // The count is read from interpreting the two bytes as a big-endian
                // uint16 value with the high-order bit masked out."

                let count = self.data.read_at::<u16>(0).unwrap_or_default() & 0x7FFF;
                // a weird case where I'm following fonttools: if the 'use words' bit
                // is set, but the total count is still 0, treat it like 0 first byte
                if count == 0 {
                    (0, 2)
                } else {
                    (count & 0x7FFF, 2)
                }
            }
        }
    }

    /// the number of bytes to encode the packed point numbers
    fn total_len(&self) -> usize {
        let (n_points, mut n_bytes) = self.count_and_count_bytes();
        if n_points == 0 {
            return n_bytes;
        }
        let mut cursor = self.data.cursor();
        cursor.advance_by(n_bytes);

        let mut n_seen = 0;
        while n_seen < n_points {
            let Some((count, two_bytes)) = read_control_byte(&mut cursor) else {
                return n_bytes;
            };
            let word_size = 1 + usize::from(two_bytes);
            let run_size = word_size * count as usize;
            n_bytes += run_size + 1; // plus the control byte;
            cursor.advance_by(run_size);
            n_seen += count as u16;
        }

        n_bytes
    }

    /// Iterate over the packed points
    pub fn iter(&self) -> PackedPointNumbersIter<'a> {
        let (count, n_bytes) = self.count_and_count_bytes();
        let mut cursor = self.data.cursor();
        cursor.advance_by(n_bytes);
        PackedPointNumbersIter::new(count, cursor)
    }
}

/// An iterator over the packed point numbers data.
#[derive(Clone, Debug)]
pub struct PackedPointNumbersIter<'a> {
    count: u16,
    seen: u16,
    last_val: u16,
    current_run: PointRunIter<'a>,
}

impl<'a> PackedPointNumbersIter<'a> {
    fn new(count: u16, cursor: Cursor<'a>) -> Self {
        PackedPointNumbersIter {
            count,
            seen: 0,
            last_val: 0,
            current_run: PointRunIter {
                remaining: 0,
                two_bytes: false,
                cursor,
            },
        }
    }
}

/// Implements the logic for iterating over the individual runs
#[derive(Clone, Debug)]
struct PointRunIter<'a> {
    remaining: u8,
    two_bytes: bool,
    cursor: Cursor<'a>,
}

impl Iterator for PointRunIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        // if no items remain in this run, start the next one.
        while self.remaining == 0 {
            (self.remaining, self.two_bytes) = read_control_byte(&mut self.cursor)?;
        }

        self.remaining -= 1;
        if self.two_bytes {
            self.cursor.read().ok()
        } else {
            self.cursor.read::<u8>().ok().map(|v| v as u16)
        }
    }
}

/// returns the count and the 'uses_two_bytes' flag from the control byte
fn read_control_byte(cursor: &mut Cursor) -> Option<(u8, bool)> {
    let control: u8 = cursor.read().ok()?;
    let two_bytes = (control & 0x80) != 0;
    let count = (control & 0x7F) + 1;
    Some((count, two_bytes))
}

impl Iterator for PackedPointNumbersIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        // if our count is zero, we keep incrementing forever
        if self.count == 0 {
            let result = self.last_val;
            self.last_val = self.last_val.checked_add(1)?;
            return Some(result);
        }

        if self.count == self.seen {
            return None;
        }
        self.seen += 1;
        self.last_val += self.current_run.next()?;
        Some(self.last_val)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count as usize, Some(self.count as usize))
    }
}

// completely unnecessary?
impl<'a> ExactSizeIterator for PackedPointNumbersIter<'a> {}

/// [Packed Deltas](https://learn.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats#packed-deltas)
#[derive(Clone, Debug)]
pub struct PackedDeltas<'a> {
    data: FontData<'a>,
    // How many values we expect
    count: usize,
}

impl<'a> PackedDeltas<'a> {
    pub(crate) fn new(data: FontData<'a>, count: usize) -> Self {
        Self { data, count }
    }

    /// NOTE: this is unbounded, and assumes all of data is deltas.
    #[doc(hidden)] // used by tests in write-fonts
    pub fn consume_all(data: FontData<'a>) -> Self {
        let count = DeltaRunIter::new(data.cursor(), None).count();
        Self { data, count }
    }

    pub(crate) fn count(&self) -> usize {
        self.count
    }

    pub fn iter(&self) -> DeltaRunIter<'a> {
        DeltaRunIter::new(self.data.cursor(), Some(self.count))
    }
}

/// Flag indicating that this run contains no data,
/// and that the deltas for this run are all zero.
const DELTAS_ARE_ZERO: u8 = 0x80;
/// Flag indicating the data type for delta values in the run.
const DELTAS_ARE_WORDS: u8 = 0x40;
/// Mask for the low 6 bits to provide the number of delta values in the run, minus one.
const DELTA_RUN_COUNT_MASK: u8 = 0x3F;

/// The type of values for a given delta run (influences the number of bytes per delta)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeltaRunType {
    Zero,
    I8,
    I16,
    I32,
}

impl DeltaRunType {
    /// The run type for a given control byte
    pub fn new(control: u8) -> Self {
        // if the top two bits of the control byte (DELTAS_ARE_ZERO and DELTAS_ARE_WORDS) are both set,
        // then the following values are 32-bit.
        // <https://github.com/harfbuzz/boring-expansion-spec/blob/main/VARC.md#tuplevalues>
        let are_zero = (control & DELTAS_ARE_ZERO) != 0;
        let are_words = (control & DELTAS_ARE_WORDS) != 0;
        match (are_zero, are_words) {
            (false, false) => Self::I8,
            (false, true) => Self::I16,
            (true, false) => Self::Zero,
            (true, true) => Self::I32,
        }
    }
}

/// Implements the logic for iterating over the individual runs
#[derive(Clone, Debug)]
pub struct DeltaRunIter<'a> {
    limit: Option<usize>, // when None, consume all available data
    remaining_in_run: u8,
    value_type: DeltaRunType,
    cursor: Cursor<'a>,
}

impl<'a> DeltaRunIter<'a> {
    fn new(cursor: Cursor<'a>, limit: Option<usize>) -> Self {
        DeltaRunIter {
            limit,
            remaining_in_run: 0,
            value_type: DeltaRunType::I8,
            cursor,
        }
    }

    pub(crate) fn end(mut self) -> Cursor<'a> {
        while self.next().is_some() {}
        self.cursor
    }
}

impl Iterator for DeltaRunIter<'_> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(limit) = self.limit {
            if limit == 0 {
                return None;
            }
            self.limit = Some(limit - 1);
        }
        // if no items remain in this run, start the next one.
        // NOTE: we use `while` so we can sanely handle the case where some
        // run in the middle of the data has an explicit zero length
        //TODO: create a font with data of this shape and go crash some font parsers
        while self.remaining_in_run == 0 {
            let control: u8 = self.cursor.read().ok()?;
            self.value_type = DeltaRunType::new(control);
            self.remaining_in_run = (control & DELTA_RUN_COUNT_MASK) + 1;
        }

        self.remaining_in_run -= 1;
        match self.value_type {
            DeltaRunType::Zero => Some(0),
            DeltaRunType::I8 => self.cursor.read::<i8>().ok().map(|v| v as i32),
            DeltaRunType::I16 => self.cursor.read::<i16>().ok().map(|v| v as i32),
            DeltaRunType::I32 => self.cursor.read().ok(),
        }
    }
}

/// A helper type for iterating over [`TupleVariationHeader`]s.
pub struct TupleVariationHeaderIter<'a> {
    data: FontData<'a>,
    n_headers: usize,
    current: usize,
    axis_count: u16,
}

impl<'a> TupleVariationHeaderIter<'a> {
    pub(crate) fn new(data: FontData<'a>, n_headers: usize, axis_count: u16) -> Self {
        Self {
            data,
            n_headers,
            current: 0,
            axis_count,
        }
    }
}

impl<'a> Iterator for TupleVariationHeaderIter<'a> {
    type Item = Result<TupleVariationHeader<'a>, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.n_headers {
            return None;
        }
        self.current += 1;
        let next = TupleVariationHeader::read(self.data, self.axis_count);
        let next_len = next
            .as_ref()
            .map(|table| table.byte_len(self.axis_count))
            .unwrap_or(0);
        self.data = self.data.split_off(next_len)?;
        Some(next)
    }
}

#[derive(Clone)]
pub struct TupleVariationData<'a, T> {
    pub(crate) axis_count: u16,
    pub(crate) shared_tuples: Option<SharedTuples<'a>>,
    pub(crate) shared_point_numbers: Option<PackedPointNumbers<'a>>,
    pub(crate) tuple_count: TupleVariationCount,
    // the data for all the tuple variation headers
    pub(crate) header_data: FontData<'a>,
    // the data for all the tuple bodies
    pub(crate) serialized_data: FontData<'a>,
    pub(crate) _marker: std::marker::PhantomData<fn() -> T>,
}

impl<'a, T> TupleVariationData<'a, T>
where
    T: TupleDelta + 'a,
{
    pub fn tuples(&self) -> TupleVariationIter<'a, T> {
        TupleVariationIter {
            current: 0,
            parent: self.clone(),
            header_iter: TupleVariationHeaderIter::new(
                self.header_data,
                self.tuple_count.count() as usize,
                self.axis_count,
            ),
            serialized_data: self.serialized_data,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns an iterator over all of the pairs of (variation tuple, scalar)
    /// for this glyph that are active for the given set of normalized
    /// coordinates.
    pub fn active_tuples_at(
        &self,
        coords: &'a [F2Dot14],
    ) -> impl Iterator<Item = (TupleVariation<'a, T>, Fixed)> + 'a {
        self.tuples().filter_map(|tuple| {
            let scaler = tuple.compute_scalar(coords)?;
            Some((tuple, scaler))
        })
    }

    pub(crate) fn tuple_count(&self) -> usize {
        self.tuple_count.count() as usize
    }
}

/// An iterator over the [`TupleVariation`]s for a specific glyph.
pub struct TupleVariationIter<'a, T> {
    current: usize,
    parent: TupleVariationData<'a, T>,
    header_iter: TupleVariationHeaderIter<'a>,
    serialized_data: FontData<'a>,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<'a, T> TupleVariationIter<'a, T>
where
    T: TupleDelta,
{
    fn next_tuple(&mut self) -> Option<TupleVariation<'a, T>> {
        if self.parent.tuple_count() == self.current {
            return None;
        }
        self.current += 1;

        // FIXME: is it okay to discard an error here?
        let header = self.header_iter.next()?.ok()?;
        let data_len = header.variation_data_size() as usize;
        let var_data = self.serialized_data.take_up_to(data_len)?;

        let (point_numbers, packed_deltas) = if header.tuple_index().private_point_numbers() {
            PackedPointNumbers::split_off_front(var_data)
        } else {
            (self.parent.shared_point_numbers.clone()?, var_data)
        };
        Some(TupleVariation {
            axis_count: self.parent.axis_count,
            header,
            shared_tuples: self.parent.shared_tuples.clone(),
            packed_deltas: PackedDeltas::consume_all(packed_deltas),
            point_numbers,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<'a, T> Iterator for TupleVariationIter<'a, T>
where
    T: TupleDelta,
{
    type Item = TupleVariation<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_tuple()
    }
}

/// A single set of tuple variation data
#[derive(Clone)]
pub struct TupleVariation<'a, T> {
    axis_count: u16,
    header: TupleVariationHeader<'a>,
    shared_tuples: Option<SharedTuples<'a>>,
    packed_deltas: PackedDeltas<'a>,
    point_numbers: PackedPointNumbers<'a>,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<'a, T> TupleVariation<'a, T>
where
    T: TupleDelta,
{
    /// Returns true if this tuple provides deltas for all points in a glyph.
    pub fn has_deltas_for_all_points(&self) -> bool {
        self.point_numbers.count() == 0
    }

    pub fn point_numbers(&'a self) -> PackedPointNumbersIter<'a> {
        self.point_numbers.iter()
    }

    /// Returns the 'peak' tuple for this variation
    pub fn peak(&self) -> Tuple<'a> {
        self.header
            .tuple_index()
            .tuple_records_index()
            .and_then(|idx| self.shared_tuples.as_ref()?.tuples().get(idx as usize).ok())
            .or_else(|| self.header.peak_tuple())
            .unwrap_or_default()
    }

    pub fn intermediate_start(&self) -> Option<Tuple<'a>> {
        self.header.intermediate_start_tuple()
    }

    pub fn intermediate_end(&self) -> Option<Tuple<'a>> {
        self.header.intermediate_end_tuple()
    }

    /// Compute the fixed point scalar for this tuple at the given location in
    /// variation space.
    ///
    /// The `coords` slice must be of lesser or equal length to the number of
    /// axes. If it is less, missing (trailing) axes will be assumed to have
    /// zero values.
    ///
    /// Returns `None` if this tuple is not applicable at the provided
    /// coordinates (e.g. if the resulting scalar is zero).
    pub fn compute_scalar(&self, coords: &[F2Dot14]) -> Option<Fixed> {
        const ZERO: Fixed = Fixed::ZERO;
        let mut scalar = Fixed::ONE;
        let peak = self.peak();
        let inter_start = self.header.intermediate_start_tuple();
        let inter_end = self.header.intermediate_end_tuple();
        if peak.len() != self.axis_count as usize {
            return None;
        }

        for i in 0..self.axis_count {
            let i = i as usize;
            let coord = coords.get(i).copied().unwrap_or_default().to_fixed();
            let peak = peak.get(i).unwrap_or_default().to_fixed();
            if peak == ZERO || peak == coord {
                continue;
            }

            if coord == ZERO {
                return None;
            }

            if let (Some(inter_start), Some(inter_end)) = (&inter_start, &inter_end) {
                let start = inter_start.get(i).unwrap_or_default().to_fixed();
                let end = inter_end.get(i).unwrap_or_default().to_fixed();
                if coord <= start || coord >= end {
                    return None;
                }
                if coord < peak {
                    scalar = scalar.mul_div(coord - start, peak - start);
                } else {
                    scalar = scalar.mul_div(end - coord, end - peak);
                }
            } else {
                if coord < peak.min(ZERO) || coord > peak.max(ZERO) {
                    return None;
                }
                scalar = scalar.mul_div(coord, peak);
            }
        }
        Some(scalar)
    }

    /// Compute the floating point scalar for this tuple at the given location
    /// in variation space.
    ///
    /// The `coords` slice must be of lesser or equal length to the number of
    /// axes. If it is less, missing (trailing) axes will be assumed to have
    /// zero values.
    ///
    /// Returns `None` if this tuple is not applicable at the provided
    /// coordinates (e.g. if the resulting scalar is zero).
    pub fn compute_scalar_f32(&self, coords: &[F2Dot14]) -> Option<f32> {
        let mut scalar = 1.0;
        let peak = self.peak();
        let inter_start = self.header.intermediate_start_tuple();
        let inter_end = self.header.intermediate_end_tuple();
        if peak.len() != self.axis_count as usize {
            return None;
        }
        for i in 0..self.axis_count {
            let i = i as usize;
            let coord = coords.get(i).copied().unwrap_or_default().to_bits() as i32;
            let peak = peak.get(i).unwrap_or_default().to_bits() as i32;
            if peak == 0 || peak == coord {
                continue;
            }
            if coord == 0 {
                return None;
            }
            if let (Some(inter_start), Some(inter_end)) = (&inter_start, &inter_end) {
                let start = inter_start.get(i).unwrap_or_default().to_bits() as i32;
                let end = inter_end.get(i).unwrap_or_default().to_bits() as i32;
                if start > peak || peak > end || (start < 0 && end > 0 && peak != 0) {
                    continue;
                }
                if coord < start || coord > end {
                    return None;
                }
                if coord < peak {
                    if peak != start {
                        scalar *= (coord - start) as f32 / (peak - start) as f32;
                    }
                } else if peak != end {
                    scalar *= (end - coord) as f32 / (end - peak) as f32;
                }
            } else {
                if coord < peak.min(0) || coord > peak.max(0) {
                    return None;
                }
                scalar *= coord as f32 / peak as f32;
            }
        }
        Some(scalar)
    }

    /// Iterate over the deltas for this tuple.
    ///
    /// This does not account for scaling. Returns only explicitly encoded
    /// deltas, e.g. an omission by IUP will not be present.
    pub fn deltas(&'a self) -> TupleDeltaIter<'a, T> {
        TupleDeltaIter::new(&self.point_numbers, &self.packed_deltas)
    }
}

/// An iterator over the deltas for a glyph.
#[derive(Clone, Debug)]
pub struct TupleDeltaIter<'a, T> {
    pub cur: usize,
    // if None all points get deltas, if Some specifies subset of points that do
    points: Option<PackedPointNumbersIter<'a>>,
    next_point: usize,
    x_iter: DeltaRunIter<'a>,
    y_iter: Option<Skip<DeltaRunIter<'a>>>,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<'a, T> TupleDeltaIter<'a, T>
where
    T: TupleDelta,
{
    fn new(points: &'a PackedPointNumbers, deltas: &'a PackedDeltas) -> TupleDeltaIter<'a, T> {
        let mut points = points.iter();
        let next_point = points.next();
        let num_encoded_points = deltas.count() / 2; // x and y encoded independently
        let y_iter = T::is_point().then(|| deltas.iter().skip(num_encoded_points));
        TupleDeltaIter {
            cur: 0,
            points: next_point.map(|_| points),
            next_point: next_point.unwrap_or_default() as usize,
            x_iter: deltas.iter(),
            y_iter,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Trait for deltas that are computed in a tuple variation store.
pub trait TupleDelta: Sized + Copy {
    /// Returns true if the delta is a point and requires reading two values
    /// from the packed delta stream.
    fn is_point() -> bool;

    /// Creates a new delta for the given position and coordinates. If
    /// the delta is not a point, the y value will always be zero.
    fn new(position: u16, x: i32, y: i32) -> Self;
}

impl<'a, T> Iterator for TupleDeltaIter<'a, T>
where
    T: TupleDelta,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let (position, dx, dy) = loop {
            let position = if let Some(points) = &mut self.points {
                // if we have points then result is sparse; only some points have deltas
                if self.cur > self.next_point {
                    self.next_point = points.next()? as usize;
                }
                self.next_point
            } else {
                // no points, every point has a delta. Just take the next one.
                self.cur
            };
            if position == self.cur {
                let dx = self.x_iter.next()?;
                let dy = if let Some(y_iter) = self.y_iter.as_mut() {
                    y_iter.next()?
                } else {
                    0
                };
                break (position, dx, dy);
            }
            self.cur += 1;
        };
        self.cur += 1;
        Some(T::new(position as u16, dx, dy))
    }
}

impl EntryFormat {
    pub fn entry_size(self) -> u8 {
        ((self.bits() & Self::MAP_ENTRY_SIZE_MASK.bits()) >> 4) + 1
    }

    pub fn bit_count(self) -> u8 {
        (self.bits() & Self::INNER_INDEX_BIT_COUNT_MASK.bits()) + 1
    }

    // called from codegen
    pub(crate) fn map_size(self, map_count: impl Into<u32>) -> usize {
        self.entry_size() as usize * map_count.into() as usize
    }
}

impl<'a> DeltaSetIndexMap<'a> {
    /// Returns the delta set index for the specified value.
    pub fn get(&self, index: u32) -> Result<DeltaSetIndex, ReadError> {
        let (entry_format, map_count, data) = match self {
            Self::Format0(fmt) => (fmt.entry_format(), fmt.map_count() as u32, fmt.map_data()),
            Self::Format1(fmt) => (fmt.entry_format(), fmt.map_count(), fmt.map_data()),
        };
        let entry_size = entry_format.entry_size();
        let data = FontData::new(data);
        // "if an index into the mapping array is used that is greater than or equal to
        // mapCount, then the last logical entry of the mapping array is used."
        // https://learn.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats
        // #associating-target-items-to-variation-data
        let index = index.min(map_count.saturating_sub(1));
        let offset = index as usize * entry_size as usize;
        let entry = match entry_size {
            1 => data.read_at::<u8>(offset)? as u32,
            2 => data.read_at::<u16>(offset)? as u32,
            3 => data.read_at::<Uint24>(offset)?.into(),
            4 => data.read_at::<u32>(offset)?,
            _ => {
                return Err(ReadError::MalformedData(
                    "invalid entry size in DeltaSetIndexMap",
                ))
            }
        };
        let bit_count = entry_format.bit_count();
        Ok(DeltaSetIndex {
            outer: (entry >> bit_count) as u16,
            inner: (entry & ((1 << bit_count) - 1)) as u16,
        })
    }
}

impl<'a> ItemVariationStore<'a> {
    /// Computes the delta value for the specified index and set of normalized
    /// variation coordinates.
    pub fn compute_delta(
        &self,
        index: DeltaSetIndex,
        coords: &[F2Dot14],
    ) -> Result<i32, ReadError> {
        let data = match self.item_variation_data().get(index.outer as usize) {
            Some(data) => data?,
            None => return Ok(0),
        };
        let regions = self.variation_region_list()?.variation_regions();
        let region_indices = data.region_indexes();
        // Compute deltas with 64-bit precision.
        // See <https://gitlab.freedesktop.org/freetype/freetype/-/blob/7ab541a2/src/truetype/ttgxvar.c#L1094>
        let mut accum = 0i64;
        for (i, region_delta) in data.delta_set(index.inner).enumerate() {
            let region_index = region_indices
                .get(i)
                .ok_or(ReadError::MalformedData(
                    "invalid delta sets in ItemVariationStore",
                ))?
                .get() as usize;
            let region = regions.get(region_index)?;
            let scalar = region.compute_scalar(coords);
            accum += region_delta as i64 * scalar.to_bits() as i64;
        }
        Ok(((accum + 0x8000) >> 16) as i32)
    }

    /// Computes the delta value in floating point for the specified index and set
    /// of normalized variation coordinates.
    pub fn compute_float_delta(
        &self,
        index: DeltaSetIndex,
        coords: &[F2Dot14],
    ) -> Result<FloatItemDelta, ReadError> {
        let data = match self.item_variation_data().get(index.outer as usize) {
            Some(data) => data?,
            None => return Ok(FloatItemDelta::ZERO),
        };
        let regions = self.variation_region_list()?.variation_regions();
        let region_indices = data.region_indexes();
        // Compute deltas in 64-bit floating point.
        let mut accum = 0f64;
        for (i, region_delta) in data.delta_set(index.inner).enumerate() {
            let region_index = region_indices
                .get(i)
                .ok_or(ReadError::MalformedData(
                    "invalid delta sets in ItemVariationStore",
                ))?
                .get() as usize;
            let region = regions.get(region_index)?;
            let scalar = region.compute_scalar_f32(coords);
            accum += region_delta as f64 * scalar as f64;
        }
        Ok(FloatItemDelta(accum))
    }
}

/// Floating point item delta computed by an item variation store.
///
/// These can be applied to types that implement [`FloatItemDeltaTarget`].
#[derive(Copy, Clone, Default, Debug)]
pub struct FloatItemDelta(f64);

impl FloatItemDelta {
    pub const ZERO: Self = Self(0.0);
}

/// Trait for applying floating point item deltas to target values.
pub trait FloatItemDeltaTarget {
    fn apply_float_delta(&self, delta: FloatItemDelta) -> f32;
}

impl FloatItemDeltaTarget for Fixed {
    fn apply_float_delta(&self, delta: FloatItemDelta) -> f32 {
        const FIXED_TO_FLOAT: f64 = 1.0 / 65536.0;
        self.to_f32() + (delta.0 * FIXED_TO_FLOAT) as f32
    }
}

impl FloatItemDeltaTarget for FWord {
    fn apply_float_delta(&self, delta: FloatItemDelta) -> f32 {
        self.to_i16() as f32 + delta.0 as f32
    }
}

impl FloatItemDeltaTarget for UfWord {
    fn apply_float_delta(&self, delta: FloatItemDelta) -> f32 {
        self.to_u16() as f32 + delta.0 as f32
    }
}

impl FloatItemDeltaTarget for F2Dot14 {
    fn apply_float_delta(&self, delta: FloatItemDelta) -> f32 {
        const F2DOT14_TO_FLOAT: f64 = 1.0 / 16384.0;
        self.to_f32() + (delta.0 * F2DOT14_TO_FLOAT) as f32
    }
}

impl<'a> VariationRegion<'a> {
    /// Computes a scalar value for this region and the specified
    /// normalized variation coordinates.
    pub fn compute_scalar(&self, coords: &[F2Dot14]) -> Fixed {
        const ZERO: Fixed = Fixed::ZERO;
        let mut scalar = Fixed::ONE;
        for (i, axis_coords) in self.region_axes().iter().enumerate() {
            let coord = coords.get(i).map(|coord| coord.to_fixed()).unwrap_or(ZERO);
            let start = axis_coords.start_coord.get().to_fixed();
            let end = axis_coords.end_coord.get().to_fixed();
            let peak = axis_coords.peak_coord.get().to_fixed();
            if start > peak || peak > end || peak == ZERO || start < ZERO && end > ZERO {
                continue;
            } else if coord < start || coord > end {
                return ZERO;
            } else if coord == peak {
                continue;
            } else if coord < peak {
                scalar = scalar.mul_div(coord - start, peak - start);
            } else {
                scalar = scalar.mul_div(end - coord, end - peak);
            }
        }
        scalar
    }

    /// Computes a floating point scalar value for this region and the
    /// specified normalized variation coordinates.
    pub fn compute_scalar_f32(&self, coords: &[F2Dot14]) -> f32 {
        let mut scalar = 1.0;
        for (i, axis_coords) in self.region_axes().iter().enumerate() {
            let coord = coords.get(i).map(|coord| coord.to_f32()).unwrap_or(0.0);
            let start = axis_coords.start_coord.get().to_f32();
            let end = axis_coords.end_coord.get().to_f32();
            let peak = axis_coords.peak_coord.get().to_f32();
            if start > peak || peak > end || peak == 0.0 || start < 0.0 && end > 0.0 {
                continue;
            } else if coord < start || coord > end {
                return 0.0;
            } else if coord == peak {
                continue;
            } else if coord < peak {
                scalar = (scalar * (coord - start)) / (peak - start);
            } else {
                scalar = (scalar * (end - coord)) / (end - peak);
            }
        }
        scalar
    }
}

impl<'a> ItemVariationData<'a> {
    /// Returns an iterator over the per-region delta values for the specified
    /// inner index.
    pub fn delta_set(&self, inner_index: u16) -> impl Iterator<Item = i32> + 'a + Clone {
        let word_delta_count = self.word_delta_count();
        let region_count = self.region_index_count();
        let bytes_per_row = Self::delta_row_len(word_delta_count, region_count);
        let long_words = word_delta_count & 0x8000 != 0;
        let word_delta_count = word_delta_count & 0x7FFF;

        let offset = bytes_per_row * inner_index as usize;
        ItemDeltas {
            cursor: FontData::new(self.delta_sets())
                .slice(offset..)
                .unwrap_or_default()
                .cursor(),
            word_delta_count,
            long_words,
            len: region_count,
            pos: 0,
        }
    }

    /// the length of one delta set
    fn delta_row_len(word_delta_count: u16, region_index_count: u16) -> usize {
        let region_count = region_index_count as usize;
        let long_words = word_delta_count & 0x8000 != 0;
        let (word_size, small_size) = if long_words { (4, 2) } else { (2, 1) };
        let long_delta_count = (word_delta_count & 0x7FFF) as usize;
        let short_delta_count = region_count.saturating_sub(long_delta_count);
        long_delta_count * word_size + short_delta_count * small_size
    }

    // called from generated code: compute the length in bytes of the delta_sets data
    fn delta_sets_len(item_count: u16, word_delta_count: u16, region_index_count: u16) -> usize {
        let bytes_per_row = Self::delta_row_len(word_delta_count, region_index_count);
        bytes_per_row * item_count as usize
    }
}

#[derive(Clone)]
struct ItemDeltas<'a> {
    cursor: Cursor<'a>,
    word_delta_count: u16,
    long_words: bool,
    len: u16,
    pos: u16,
}

impl<'a> Iterator for ItemDeltas<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        let pos = self.pos;
        self.pos += 1;
        let value = match (pos >= self.word_delta_count, self.long_words) {
            (true, true) | (false, false) => self.cursor.read::<i16>().ok()? as i32,
            (true, false) => self.cursor.read::<i8>().ok()? as i32,
            (false, true) => self.cursor.read::<i32>().ok()?,
        };
        Some(value)
    }
}

pub(crate) fn advance_delta(
    dsim: Option<Result<DeltaSetIndexMap, ReadError>>,
    ivs: Result<ItemVariationStore, ReadError>,
    glyph_id: GlyphId,
    coords: &[F2Dot14],
) -> Result<Fixed, ReadError> {
    let gid = glyph_id.to_u16();
    let ix = match dsim {
        Some(Ok(dsim)) => dsim.get(gid as u32)?,
        _ => DeltaSetIndex {
            outer: 0,
            inner: gid,
        },
    };
    Ok(Fixed::from_i32(ivs?.compute_delta(ix, coords)?))
}

pub(crate) fn item_delta(
    dsim: Option<Result<DeltaSetIndexMap, ReadError>>,
    ivs: Result<ItemVariationStore, ReadError>,
    glyph_id: GlyphId,
    coords: &[F2Dot14],
) -> Result<Fixed, ReadError> {
    let gid = glyph_id.to_u16();
    let ix = match dsim {
        Some(Ok(dsim)) => dsim.get(gid as u32)?,
        _ => return Err(ReadError::NullOffset),
    };
    Ok(Fixed::from_i32(ivs?.compute_delta(ix, coords)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_helpers::BeBuffer, FontRef, TableProvider};

    #[test]
    fn ivs_regions() {
        let font = FontRef::new(font_test_data::VAZIRMATN_VAR).unwrap();
        let hvar = font.hvar().expect("missing HVAR table");
        let ivs = hvar
            .item_variation_store()
            .expect("missing item variation store in HVAR");
        let region_list = ivs.variation_region_list().expect("missing region list!");
        let regions = region_list.variation_regions();
        let expected = &[
            // start_coord, peak_coord, end_coord
            vec![[-1.0f32, -1.0, 0.0]],
            vec![[0.0, 1.0, 1.0]],
        ][..];
        let region_coords = regions
            .iter()
            .map(|region| {
                region
                    .unwrap()
                    .region_axes()
                    .iter()
                    .map(|coords| {
                        [
                            coords.start_coord().to_f32(),
                            coords.peak_coord().to_f32(),
                            coords.end_coord().to_f32(),
                        ]
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(expected, &region_coords);
    }

    // adapted from https://github.com/fonttools/fonttools/blob/f73220816264fc383b8a75f2146e8d69e455d398/Tests/ttLib/tables/TupleVariation_test.py#L492
    #[test]
    fn packed_points() {
        fn decode_points(bytes: &[u8]) -> Option<Vec<u16>> {
            let data = FontData::new(bytes);
            let packed = PackedPointNumbers { data };
            if packed.count() == 0 {
                None
            } else {
                Some(packed.iter().collect())
            }
        }

        assert_eq!(decode_points(&[0]), None);
        // all points in glyph (in overly verbose encoding, not explicitly prohibited by spec)
        assert_eq!(decode_points(&[0x80, 0]), None);
        // 2 points; first run: [9, 9+6]
        assert_eq!(decode_points(&[0x02, 0x01, 0x09, 0x06]), Some(vec![9, 15]));
        // 2 points; first run: [0xBEEF, 0xCAFE]. (0x0C0F = 0xCAFE - 0xBEEF)
        assert_eq!(
            decode_points(&[0x02, 0x81, 0xbe, 0xef, 0x0c, 0x0f]),
            Some(vec![0xbeef, 0xcafe])
        );
        // 1 point; first run: [7]
        assert_eq!(decode_points(&[0x01, 0, 0x07]), Some(vec![7]));
        // 1 point; first run: [7] in overly verbose encoding
        assert_eq!(decode_points(&[0x01, 0x80, 0, 0x07]), Some(vec![7]));
        // 1 point; first run: [65535]; requires words to be treated as unsigned numbers
        assert_eq!(decode_points(&[0x01, 0x80, 0xff, 0xff]), Some(vec![65535]));
        // 4 points; first run: [7, 8]; second run: [255, 257]. 257 is stored in delta-encoded bytes (0xFF + 2).
        assert_eq!(
            decode_points(&[0x04, 1, 7, 1, 1, 0xff, 2]),
            Some(vec![7, 8, 263, 265])
        );
    }

    #[test]
    fn packed_point_byte_len() {
        fn count_bytes(bytes: &[u8]) -> usize {
            let packed = PackedPointNumbers {
                data: FontData::new(bytes),
            };
            packed.total_len()
        }

        static CASES: &[&[u8]] = &[
            &[0],
            &[0x80, 0],
            &[0x02, 0x01, 0x09, 0x06],
            &[0x02, 0x81, 0xbe, 0xef, 0x0c, 0x0f],
            &[0x01, 0, 0x07],
            &[0x01, 0x80, 0, 0x07],
            &[0x01, 0x80, 0xff, 0xff],
            &[0x04, 1, 7, 1, 1, 0xff, 2],
        ];

        for case in CASES {
            assert_eq!(count_bytes(case), case.len(), "{case:?}");
        }
    }

    // https://github.com/fonttools/fonttools/blob/c30a6355ffdf7f09d31e7719975b4b59bac410af/Tests/ttLib/tables/TupleVariation_test.py#L670
    #[test]
    fn packed_deltas() {
        static INPUT: FontData = FontData::new(&[0x83, 0x40, 0x01, 0x02, 0x01, 0x81, 0x80]);

        let deltas = PackedDeltas::consume_all(INPUT);
        assert_eq!(deltas.count, 7);
        assert_eq!(
            deltas.iter().collect::<Vec<_>>(),
            &[0, 0, 0, 0, 258, -127, -128]
        );

        assert_eq!(
            PackedDeltas::consume_all(FontData::new(&[0x81]))
                .iter()
                .collect::<Vec<_>>(),
            &[0, 0,]
        );
    }

    // https://learn.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats#packed-deltas
    #[test]
    fn packed_deltas_spec() {
        static INPUT: FontData = FontData::new(&[
            0x03, 0x0A, 0x97, 0x00, 0xC6, 0x87, 0x41, 0x10, 0x22, 0xFB, 0x34,
        ]);
        static EXPECTED: &[i32] = &[10, -105, 0, -58, 0, 0, 0, 0, 0, 0, 0, 0, 4130, -1228];

        let deltas = PackedDeltas::consume_all(INPUT);
        assert_eq!(deltas.count, EXPECTED.len());
        assert_eq!(deltas.iter().collect::<Vec<_>>(), EXPECTED);
    }

    #[test]
    fn packed_point_split() {
        static INPUT: FontData =
            FontData::new(&[2, 1, 1, 2, 1, 205, 143, 1, 8, 0, 1, 202, 59, 1, 255, 0]);
        let (points, data) = PackedPointNumbers::split_off_front(INPUT);
        assert_eq!(points.count(), 2);
        assert_eq!(points.iter().collect::<Vec<_>>(), &[1, 3]);
        assert_eq!(points.total_len(), 4);
        assert_eq!(data.len(), INPUT.len() - 4);
    }

    #[test]
    fn packed_points_dont_panic() {
        // a single '0' byte means that there are deltas for all points
        static ALL_POINTS: FontData = FontData::new(&[0]);
        let (all_points, _) = PackedPointNumbers::split_off_front(ALL_POINTS);
        // in which case the iterator just keeps incrementing until u16::MAX
        assert_eq!(all_points.iter().count(), u16::MAX as _);
    }

    /// We don't have a reference for our float delta computation, so this is
    /// a sanity test to ensure that floating point deltas are within a
    /// reasonable margin of the same in fixed point.
    #[test]
    fn ivs_float_deltas_nearly_match_fixed_deltas() {
        let font = FontRef::new(font_test_data::COLRV0V1_VARIABLE).unwrap();
        let axis_count = font.fvar().unwrap().axis_count() as usize;
        let colr = font.colr().unwrap();
        let ivs = colr.item_variation_store().unwrap().unwrap();
        // Generate a set of coords from -1 to 1 in 0.1 increments
        for coord in (0..=20).map(|x| F2Dot14::from_f32((x as f32) / 10.0 - 1.0)) {
            // For testing purposes, just splat the coord to all axes
            let coords = vec![coord; axis_count];
            for (outer_ix, data) in ivs.item_variation_data().iter().enumerate() {
                let outer_ix = outer_ix as u16;
                let Some(Ok(data)) = data else {
                    continue;
                };
                for inner_ix in 0..data.item_count() {
                    let delta_ix = DeltaSetIndex {
                        outer: outer_ix,
                        inner: inner_ix,
                    };
                    // Check the deltas against all possible target values
                    let orig_delta = ivs.compute_delta(delta_ix, &coords).unwrap();
                    let float_delta = ivs.compute_float_delta(delta_ix, &coords).unwrap();
                    // For font unit types, we need to accept both rounding and
                    // truncation to account for the additional accumulation of
                    // fractional bits in floating point
                    assert!(
                        orig_delta == float_delta.0.round() as i32
                            || orig_delta == float_delta.0.trunc() as i32
                    );
                    // For the fixed point types, check with an epsilon
                    const EPSILON: f32 = 1e12;
                    let fixed_delta = Fixed::ZERO.apply_float_delta(float_delta);
                    assert!((Fixed::from_bits(orig_delta).to_f32() - fixed_delta).abs() < EPSILON);
                    let f2dot14_delta = F2Dot14::ZERO.apply_float_delta(float_delta);
                    assert!(
                        (F2Dot14::from_bits(orig_delta as i16).to_f32() - f2dot14_delta).abs()
                            < EPSILON
                    );
                }
            }
        }
    }

    #[test]
    fn ivs_data_len_short() {
        let data = BeBuffer::new()
            .push(2u16) // item_count
            .push(3u16) // word_delta_count
            .push(5u16) // region_index_count
            .extend([0u16, 1, 2, 3, 4]) // region_indices
            .extend([1u8; 128]); // this is much more data than we need!

        let ivs = ItemVariationData::read(data.font_data()).unwrap();
        let row_len = (3 * u16::RAW_BYTE_LEN) + (2 * u8::RAW_BYTE_LEN); // 3 word deltas, 2 byte deltas
        let expected_len = 2 * row_len;
        assert_eq!(ivs.delta_sets().len(), expected_len);
    }

    #[test]
    fn ivs_data_len_long() {
        let data = BeBuffer::new()
            .push(2u16) // item_count
            .push(2u16 | 0x8000) // word_delta_count, long deltas
            .push(4u16) // region_index_count
            .extend([0u16, 1, 2]) // region_indices
            .extend([1u8; 128]); // this is much more data than we need!

        let ivs = ItemVariationData::read(data.font_data()).unwrap();
        let row_len = (2 * u32::RAW_BYTE_LEN) + (2 * u16::RAW_BYTE_LEN); // 1 word (4-byte) delta, 2 short (2-byte)
        let expected_len = 2 * row_len;
        assert_eq!(ivs.delta_sets().len(), expected_len);
    }
}
