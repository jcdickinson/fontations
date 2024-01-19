// THIS FILE IS AUTOGENERATED.
// Any changes to this file will be overwritten.
// For more information about how codegen works, see font-codegen/README.md

#[allow(unused_imports)]
use crate::codegen_prelude::*;

/// The [glyf (Glyph Data)](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) table
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct GlyfMarker {}

impl GlyfMarker {}

impl TopLevelTable for Glyf<'_> {
    /// `glyf`
    const TAG: Tag = Tag::new(b"glyf");
}

impl<'a> FontRead<'a> for Glyf<'a> {
    fn read(data: FontData<'a>) -> Result<Self, ReadError> {
        let cursor = data.cursor();
        cursor.finish(GlyfMarker {})
    }
}

/// The [glyf (Glyph Data)](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) table
pub type Glyf<'a> = TableRef<'a, GlyfMarker>;

impl<'a> Glyf<'a> {}

#[cfg(feature = "traversal")]
impl<'a> SomeTable<'a> for Glyf<'a> {
    fn type_name(&self) -> &str {
        "Glyf"
    }

    #[allow(unused_variables)]
    #[allow(clippy::match_single_binding)]
    fn get_field(&self, idx: usize) -> Option<Field<'a>> {
        match idx {
            _ => None,
        }
    }
}

#[cfg(feature = "traversal")]
impl<'a> std::fmt::Debug for Glyf<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn SomeTable<'a>).fmt(f)
    }
}

/// The [Glyph Header](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct SimpleGlyphMarker {
    end_pts_of_contours_byte_len: usize,
    instructions_byte_len: usize,
    glyph_data_byte_len: usize,
}

impl SimpleGlyphMarker {
    fn number_of_contours_byte_range(&self) -> Range<usize> {
        let start = 0;
        start..start + i16::RAW_BYTE_LEN
    }
    fn x_min_byte_range(&self) -> Range<usize> {
        let start = self.number_of_contours_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn y_min_byte_range(&self) -> Range<usize> {
        let start = self.x_min_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn x_max_byte_range(&self) -> Range<usize> {
        let start = self.y_min_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn y_max_byte_range(&self) -> Range<usize> {
        let start = self.x_max_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn end_pts_of_contours_byte_range(&self) -> Range<usize> {
        let start = self.y_max_byte_range().end;
        start..start + self.end_pts_of_contours_byte_len
    }
    fn instruction_length_byte_range(&self) -> Range<usize> {
        let start = self.end_pts_of_contours_byte_range().end;
        start..start + u16::RAW_BYTE_LEN
    }
    fn instructions_byte_range(&self) -> Range<usize> {
        let start = self.instruction_length_byte_range().end;
        start..start + self.instructions_byte_len
    }
    fn glyph_data_byte_range(&self) -> Range<usize> {
        let start = self.instructions_byte_range().end;
        start..start + self.glyph_data_byte_len
    }
}

impl<'a> FontRead<'a> for SimpleGlyph<'a> {
    fn read(data: FontData<'a>) -> Result<Self, ReadError> {
        let mut cursor = data.cursor();
        let number_of_contours: i16 = cursor.read()?;
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        let end_pts_of_contours_byte_len = number_of_contours as usize * u16::RAW_BYTE_LEN;
        cursor.advance_by(end_pts_of_contours_byte_len);
        let instruction_length: u16 = cursor.read()?;
        let instructions_byte_len = instruction_length as usize * u8::RAW_BYTE_LEN;
        cursor.advance_by(instructions_byte_len);
        let glyph_data_byte_len = cursor.remaining_bytes();
        cursor.advance_by(glyph_data_byte_len);
        cursor.finish(SimpleGlyphMarker {
            end_pts_of_contours_byte_len,
            instructions_byte_len,
            glyph_data_byte_len,
        })
    }
}

/// The [Glyph Header](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
pub type SimpleGlyph<'a> = TableRef<'a, SimpleGlyphMarker>;

impl<'a> SimpleGlyph<'a> {
    /// If the number of contours is greater than or equal to zero,
    /// this is a simple glyph. If negative, this is a composite glyph
    /// — the value -1 should be used for composite glyphs.
    pub fn number_of_contours(&self) -> i16 {
        let range = self.shape.number_of_contours_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Minimum x for coordinate data.
    pub fn x_min(&self) -> i16 {
        let range = self.shape.x_min_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Minimum y for coordinate data.
    pub fn y_min(&self) -> i16 {
        let range = self.shape.y_min_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Maximum x for coordinate data.
    pub fn x_max(&self) -> i16 {
        let range = self.shape.x_max_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Maximum y for coordinate data.
    pub fn y_max(&self) -> i16 {
        let range = self.shape.y_max_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Array of point indices for the last point of each contour,
    /// in increasing numeric order
    pub fn end_pts_of_contours(&self) -> &'a [BigEndian<u16>] {
        let range = self.shape.end_pts_of_contours_byte_range();
        self.data.read_array(range).unwrap()
    }

    /// Total number of bytes for instructions. If instructionLength is
    /// zero, no instructions are present for this glyph, and this
    /// field is followed directly by the flags field.
    pub fn instruction_length(&self) -> u16 {
        let range = self.shape.instruction_length_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Array of instruction byte code for the glyph.
    pub fn instructions(&self) -> &'a [u8] {
        let range = self.shape.instructions_byte_range();
        self.data.read_array(range).unwrap()
    }

    /// the raw data for flags & x/y coordinates
    pub fn glyph_data(&self) -> &'a [u8] {
        let range = self.shape.glyph_data_byte_range();
        self.data.read_array(range).unwrap()
    }
}

#[cfg(feature = "traversal")]
impl<'a> SomeTable<'a> for SimpleGlyph<'a> {
    fn type_name(&self) -> &str {
        "SimpleGlyph"
    }
    fn get_field(&self, idx: usize) -> Option<Field<'a>> {
        match idx {
            0usize => Some(Field::new("number_of_contours", self.number_of_contours())),
            1usize => Some(Field::new("x_min", self.x_min())),
            2usize => Some(Field::new("y_min", self.y_min())),
            3usize => Some(Field::new("x_max", self.x_max())),
            4usize => Some(Field::new("y_max", self.y_max())),
            5usize => Some(Field::new(
                "end_pts_of_contours",
                self.end_pts_of_contours(),
            )),
            6usize => Some(Field::new("instruction_length", self.instruction_length())),
            7usize => Some(Field::new("instructions", self.instructions())),
            8usize => Some(Field::new("glyph_data", self.glyph_data())),
            _ => None,
        }
    }
}

#[cfg(feature = "traversal")]
impl<'a> std::fmt::Debug for SimpleGlyph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn SomeTable<'a>).fmt(f)
    }
}

/// Flags used in [SimpleGlyph]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimpleGlyphFlags {
    bits: u8,
}

impl SimpleGlyphFlags {
    /// Bit 0: If set, the point is on the curve; otherwise, it is off
    /// the curve.
    pub const ON_CURVE_POINT: Self = Self { bits: 0x01 };

    /// Bit 1: If set, the corresponding x-coordinate is 1 byte long,
    /// and the sign is determined by the
    /// X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag. If not set, its
    /// interpretation depends on the
    /// X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag: If that other flag
    /// is set, the x-coordinate is the same as the previous
    /// x-coordinate, and no element is added to the xCoordinates
    /// array. If both flags are not set, the corresponding element in
    /// the xCoordinates array is two bytes and interpreted as a signed
    /// integer. See the description of the
    /// X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag for additional
    /// information.
    pub const X_SHORT_VECTOR: Self = Self { bits: 0x02 };

    /// Bit 2: If set, the corresponding y-coordinate is 1 byte long,
    /// and the sign is determined by the
    /// Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag. If not set, its
    /// interpretation depends on the
    /// Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag: If that other flag
    /// is set, the y-coordinate is the same as the previous
    /// y-coordinate, and no element is added to the yCoordinates
    /// array. If both flags are not set, the corresponding element in
    /// the yCoordinates array is two bytes and interpreted as a signed
    /// integer. See the description of the
    /// Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag for additional
    /// information.
    pub const Y_SHORT_VECTOR: Self = Self { bits: 0x04 };

    /// Bit 3: If set, the next byte (read as unsigned) specifies the
    /// number of additional times this flag byte is to be repeated in
    /// the logical flags array — that is, the number of additional
    /// logical flag entries inserted after this entry. (In the
    /// expanded logical array, this bit is ignored.) In this way, the
    /// number of flags listed can be smaller than the number of points
    /// in the glyph description.
    pub const REPEAT_FLAG: Self = Self { bits: 0x08 };

    /// Bit 4: This flag has two meanings, depending on how the
    /// X_SHORT_VECTOR flag is set. If X_SHORT_VECTOR is set, this bit
    /// describes the sign of the value, with 1 equalling positive and
    /// 0 negative. If X_SHORT_VECTOR is not set and this bit is set,
    /// then the current x-coordinate is the same as the previous
    /// x-coordinate. If X_SHORT_VECTOR is not set and this bit is also
    /// not set, the current x-coordinate is a signed 16-bit delta
    /// vector.
    pub const X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR: Self = Self { bits: 0x10 };

    /// Bit 5: This flag has two meanings, depending on how the
    /// Y_SHORT_VECTOR flag is set. If Y_SHORT_VECTOR is set, this bit
    /// describes the sign of the value, with 1 equalling positive and
    /// 0 negative. If Y_SHORT_VECTOR is not set and this bit is set,
    /// then the current y-coordinate is the same as the previous
    /// y-coordinate. If Y_SHORT_VECTOR is not set and this bit is also
    /// not set, the current y-coordinate is a signed 16-bit delta
    /// vector.
    pub const Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR: Self = Self { bits: 0x20 };

    /// Bit 6: If set, contours in the glyph description may overlap.
    /// Use of this flag is not required in OpenType — that is, it is
    /// valid to have contours overlap without having this flag set. It
    /// may affect behaviors in some platforms, however. (See the
    /// discussion of “Overlapping contours” in Apple’s
    /// specification for details regarding behavior in Apple
    /// platforms.) When used, it must be set on the first flag byte
    /// for the glyph. See additional details below.
    pub const OVERLAP_SIMPLE: Self = Self { bits: 0x40 };
}

impl SimpleGlyphFlags {
    ///  Returns an empty set of flags.
    #[inline]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Returns the set containing all flags.
    #[inline]
    pub const fn all() -> Self {
        Self {
            bits: Self::ON_CURVE_POINT.bits
                | Self::X_SHORT_VECTOR.bits
                | Self::Y_SHORT_VECTOR.bits
                | Self::REPEAT_FLAG.bits
                | Self::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR.bits
                | Self::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR.bits
                | Self::OVERLAP_SIMPLE.bits,
        }
    }

    /// Returns the raw value of the flags currently stored.
    #[inline]
    pub const fn bits(&self) -> u8 {
        self.bits
    }

    /// Convert from underlying bit representation, unless that
    /// representation contains bits that do not correspond to a flag.
    #[inline]
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if (bits & !Self::all().bits()) == 0 {
            Some(Self { bits })
        } else {
            None
        }
    }

    /// Convert from underlying bit representation, dropping any bits
    /// that do not correspond to flags.
    #[inline]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self {
            bits: bits & Self::all().bits,
        }
    }

    /// Returns `true` if no flags are currently stored.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bits() == Self::empty().bits()
    }

    /// Returns `true` if there are flags common to both `self` and `other`.
    #[inline]
    pub const fn intersects(&self, other: Self) -> bool {
        !(Self {
            bits: self.bits & other.bits,
        })
        .is_empty()
    }

    /// Returns `true` if all of the flags in `other` are contained within `self`.
    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Inserts the specified flags in-place.
    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.bits |= other.bits;
    }

    /// Removes the specified flags in-place.
    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.bits &= !other.bits;
    }

    /// Toggles the specified flags in-place.
    #[inline]
    pub fn toggle(&mut self, other: Self) {
        self.bits ^= other.bits;
    }

    /// Returns the intersection between the flags in `self` and
    /// `other`.
    ///
    /// Specifically, the returned set contains only the flags which are
    /// present in *both* `self` *and* `other`.
    ///
    /// This is equivalent to using the `&` operator (e.g.
    /// [`ops::BitAnd`]), as in `flags & other`.
    ///
    /// [`ops::BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }

    /// Returns the union of between the flags in `self` and `other`.
    ///
    /// Specifically, the returned set contains all flags which are
    /// present in *either* `self` *or* `other`, including any which are
    /// present in both.
    ///
    /// This is equivalent to using the `|` operator (e.g.
    /// [`ops::BitOr`]), as in `flags | other`.
    ///
    /// [`ops::BitOr`]: https://doc.rust-lang.org/std/ops/trait.BitOr.html
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// Returns the difference between the flags in `self` and `other`.
    ///
    /// Specifically, the returned set contains all flags present in
    /// `self`, except for the ones present in `other`.
    ///
    /// It is also conceptually equivalent to the "bit-clear" operation:
    /// `flags & !other` (and this syntax is also supported).
    ///
    /// This is equivalent to using the `-` operator (e.g.
    /// [`ops::Sub`]), as in `flags - other`.
    ///
    /// [`ops::Sub`]: https://doc.rust-lang.org/std/ops/trait.Sub.html
    #[inline]
    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }
}

impl std::ops::BitOr for SimpleGlyphFlags {
    type Output = Self;

    /// Returns the union of the two sets of flags.
    #[inline]
    fn bitor(self, other: SimpleGlyphFlags) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
}

impl std::ops::BitOrAssign for SimpleGlyphFlags {
    /// Adds the set of flags.
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.bits |= other.bits;
    }
}

impl std::ops::BitXor for SimpleGlyphFlags {
    type Output = Self;

    /// Returns the left flags, but with all the right flags toggled.
    #[inline]
    fn bitxor(self, other: Self) -> Self {
        Self {
            bits: self.bits ^ other.bits,
        }
    }
}

impl std::ops::BitXorAssign for SimpleGlyphFlags {
    /// Toggles the set of flags.
    #[inline]
    fn bitxor_assign(&mut self, other: Self) {
        self.bits ^= other.bits;
    }
}

impl std::ops::BitAnd for SimpleGlyphFlags {
    type Output = Self;

    /// Returns the intersection between the two sets of flags.
    #[inline]
    fn bitand(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }
}

impl std::ops::BitAndAssign for SimpleGlyphFlags {
    /// Disables all flags disabled in the set.
    #[inline]
    fn bitand_assign(&mut self, other: Self) {
        self.bits &= other.bits;
    }
}

impl std::ops::Sub for SimpleGlyphFlags {
    type Output = Self;

    /// Returns the set difference of the two sets of flags.
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }
}

impl std::ops::SubAssign for SimpleGlyphFlags {
    /// Disables all flags enabled in the set.
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.bits &= !other.bits;
    }
}

impl std::ops::Not for SimpleGlyphFlags {
    type Output = Self;

    /// Returns the complement of this set of flags.
    #[inline]
    fn not(self) -> Self {
        Self { bits: !self.bits } & Self::all()
    }
}

impl std::fmt::Debug for SimpleGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let members: &[(&str, Self)] = &[
            ("ON_CURVE_POINT", Self::ON_CURVE_POINT),
            ("X_SHORT_VECTOR", Self::X_SHORT_VECTOR),
            ("Y_SHORT_VECTOR", Self::Y_SHORT_VECTOR),
            ("REPEAT_FLAG", Self::REPEAT_FLAG),
            (
                "X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR",
                Self::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR,
            ),
            (
                "Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR",
                Self::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR,
            ),
            ("OVERLAP_SIMPLE", Self::OVERLAP_SIMPLE),
        ];
        let mut first = true;
        for (name, value) in members {
            if self.contains(*value) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str(name)?;
            }
        }
        if first {
            f.write_str("(empty)")?;
        }
        Ok(())
    }
}

impl std::fmt::Binary for SimpleGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Binary::fmt(&self.bits, f)
    }
}

impl std::fmt::Octal for SimpleGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Octal::fmt(&self.bits, f)
    }
}

impl std::fmt::LowerHex for SimpleGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::LowerHex::fmt(&self.bits, f)
    }
}

impl std::fmt::UpperHex for SimpleGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.bits, f)
    }
}

impl font_types::Scalar for SimpleGlyphFlags {
    type Raw = <u8 as font_types::Scalar>::Raw;
    fn to_raw(self) -> Self::Raw {
        self.bits().to_raw()
    }
    fn from_raw(raw: Self::Raw) -> Self {
        let t = <u8>::from_raw(raw);
        Self::from_bits_truncate(t)
    }
}

#[cfg(feature = "traversal")]
impl<'a> From<SimpleGlyphFlags> for FieldType<'a> {
    fn from(src: SimpleGlyphFlags) -> FieldType<'a> {
        src.bits().into()
    }
}

/// [CompositeGlyph](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct CompositeGlyphMarker {
    component_data_byte_len: usize,
}

impl CompositeGlyphMarker {
    fn number_of_contours_byte_range(&self) -> Range<usize> {
        let start = 0;
        start..start + i16::RAW_BYTE_LEN
    }
    fn x_min_byte_range(&self) -> Range<usize> {
        let start = self.number_of_contours_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn y_min_byte_range(&self) -> Range<usize> {
        let start = self.x_min_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn x_max_byte_range(&self) -> Range<usize> {
        let start = self.y_min_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn y_max_byte_range(&self) -> Range<usize> {
        let start = self.x_max_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn component_data_byte_range(&self) -> Range<usize> {
        let start = self.y_max_byte_range().end;
        start..start + self.component_data_byte_len
    }
}

impl<'a> FontRead<'a> for CompositeGlyph<'a> {
    fn read(data: FontData<'a>) -> Result<Self, ReadError> {
        let mut cursor = data.cursor();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        let component_data_byte_len = cursor.remaining_bytes();
        cursor.advance_by(component_data_byte_len);
        cursor.finish(CompositeGlyphMarker {
            component_data_byte_len,
        })
    }
}

/// [CompositeGlyph](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
pub type CompositeGlyph<'a> = TableRef<'a, CompositeGlyphMarker>;

impl<'a> CompositeGlyph<'a> {
    /// If the number of contours is greater than or equal to zero,
    /// this is a simple glyph. If negative, this is a composite glyph
    /// — the value -1 should be used for composite glyphs.
    pub fn number_of_contours(&self) -> i16 {
        let range = self.shape.number_of_contours_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Minimum x for coordinate data.
    pub fn x_min(&self) -> i16 {
        let range = self.shape.x_min_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Minimum y for coordinate data.
    pub fn y_min(&self) -> i16 {
        let range = self.shape.y_min_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Maximum x for coordinate data.
    pub fn x_max(&self) -> i16 {
        let range = self.shape.x_max_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Maximum y for coordinate data.
    pub fn y_max(&self) -> i16 {
        let range = self.shape.y_max_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// component flag
    /// glyph index of component
    pub fn component_data(&self) -> &'a [u8] {
        let range = self.shape.component_data_byte_range();
        self.data.read_array(range).unwrap()
    }
}

#[cfg(feature = "traversal")]
impl<'a> SomeTable<'a> for CompositeGlyph<'a> {
    fn type_name(&self) -> &str {
        "CompositeGlyph"
    }
    fn get_field(&self, idx: usize) -> Option<Field<'a>> {
        match idx {
            0usize => Some(Field::new("number_of_contours", self.number_of_contours())),
            1usize => Some(Field::new("x_min", self.x_min())),
            2usize => Some(Field::new("y_min", self.y_min())),
            3usize => Some(Field::new("x_max", self.x_max())),
            4usize => Some(Field::new("y_max", self.y_max())),
            5usize => Some(Field::new("component_data", self.component_data())),
            _ => None,
        }
    }
}

#[cfg(feature = "traversal")]
impl<'a> std::fmt::Debug for CompositeGlyph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn SomeTable<'a>).fmt(f)
    }
}

/// Flags used in [CompositeGlyph]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompositeGlyphFlags {
    bits: u16,
}

impl CompositeGlyphFlags {
    /// Bit 0: If this is set, the arguments are 16-bit (uint16 or
    /// int16); otherwise, they are bytes (uint8 or int8).
    pub const ARG_1_AND_2_ARE_WORDS: Self = Self { bits: 0x0001 };

    /// Bit 1: If this is set, the arguments are signed xy values;

    /// otherwise, they are unsigned point numbers.
    pub const ARGS_ARE_XY_VALUES: Self = Self { bits: 0x0002 };

    /// Bit 2: If set and ARGS_ARE_XY_VALUES is also set, the xy values
    /// are rounded to the nearest grid line. Ignored if
    /// ARGS_ARE_XY_VALUES is not set.
    pub const ROUND_XY_TO_GRID: Self = Self { bits: 0x0004 };

    /// Bit 3: This indicates that there is a simple scale for the
    /// component. Otherwise, scale = 1.0.
    pub const WE_HAVE_A_SCALE: Self = Self { bits: 0x0008 };

    /// Bit 5: Indicates at least one more glyph after this one.
    pub const MORE_COMPONENTS: Self = Self { bits: 0x0020 };

    /// Bit 6: The x direction will use a different scale from the y
    /// direction.
    pub const WE_HAVE_AN_X_AND_Y_SCALE: Self = Self { bits: 0x0040 };

    /// Bit 7: There is a 2 by 2 transformation that will be used to
    /// scale the component.
    pub const WE_HAVE_A_TWO_BY_TWO: Self = Self { bits: 0x0080 };

    /// Bit 8: Following the last component are instructions for the
    /// composite character.
    pub const WE_HAVE_INSTRUCTIONS: Self = Self { bits: 0x0100 };

    /// Bit 9: If set, this forces the aw and lsb (and rsb) for the
    /// composite to be equal to those from this component glyph. This
    /// works for hinted and unhinted glyphs.
    pub const USE_MY_METRICS: Self = Self { bits: 0x0200 };

    /// Bit 10: If set, the components of the compound glyph overlap.
    /// Use of this flag is not required in OpenType — that is, it is
    /// valid to have components overlap without having this flag set.
    /// It may affect behaviors in some platforms, however. (See
    /// Apple’s specification for details regarding behavior in Apple
    /// platforms.) When used, it must be set on the flag word for the
    /// first component. See additional remarks, above, for the similar
    /// OVERLAP_SIMPLE flag used in simple-glyph descriptions.
    pub const OVERLAP_COMPOUND: Self = Self { bits: 0x0400 };

    /// Bit 11: The composite is designed to have the component offset
    /// scaled. Ignored if ARGS_ARE_XY_VALUES is not set.
    pub const SCALED_COMPONENT_OFFSET: Self = Self { bits: 0x0800 };

    /// Bit 12: The composite is designed not to have the component
    /// offset scaled. Ignored if ARGS_ARE_XY_VALUES is not set.
    pub const UNSCALED_COMPONENT_OFFSET: Self = Self { bits: 0x1000 };
}

impl CompositeGlyphFlags {
    ///  Returns an empty set of flags.
    #[inline]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Returns the set containing all flags.
    #[inline]
    pub const fn all() -> Self {
        Self {
            bits: Self::ARG_1_AND_2_ARE_WORDS.bits
                | Self::ARGS_ARE_XY_VALUES.bits
                | Self::ROUND_XY_TO_GRID.bits
                | Self::WE_HAVE_A_SCALE.bits
                | Self::MORE_COMPONENTS.bits
                | Self::WE_HAVE_AN_X_AND_Y_SCALE.bits
                | Self::WE_HAVE_A_TWO_BY_TWO.bits
                | Self::WE_HAVE_INSTRUCTIONS.bits
                | Self::USE_MY_METRICS.bits
                | Self::OVERLAP_COMPOUND.bits
                | Self::SCALED_COMPONENT_OFFSET.bits
                | Self::UNSCALED_COMPONENT_OFFSET.bits,
        }
    }

    /// Returns the raw value of the flags currently stored.
    #[inline]
    pub const fn bits(&self) -> u16 {
        self.bits
    }

    /// Convert from underlying bit representation, unless that
    /// representation contains bits that do not correspond to a flag.
    #[inline]
    pub const fn from_bits(bits: u16) -> Option<Self> {
        if (bits & !Self::all().bits()) == 0 {
            Some(Self { bits })
        } else {
            None
        }
    }

    /// Convert from underlying bit representation, dropping any bits
    /// that do not correspond to flags.
    #[inline]
    pub const fn from_bits_truncate(bits: u16) -> Self {
        Self {
            bits: bits & Self::all().bits,
        }
    }

    /// Returns `true` if no flags are currently stored.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bits() == Self::empty().bits()
    }

    /// Returns `true` if there are flags common to both `self` and `other`.
    #[inline]
    pub const fn intersects(&self, other: Self) -> bool {
        !(Self {
            bits: self.bits & other.bits,
        })
        .is_empty()
    }

    /// Returns `true` if all of the flags in `other` are contained within `self`.
    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Inserts the specified flags in-place.
    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.bits |= other.bits;
    }

    /// Removes the specified flags in-place.
    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.bits &= !other.bits;
    }

    /// Toggles the specified flags in-place.
    #[inline]
    pub fn toggle(&mut self, other: Self) {
        self.bits ^= other.bits;
    }

    /// Returns the intersection between the flags in `self` and
    /// `other`.
    ///
    /// Specifically, the returned set contains only the flags which are
    /// present in *both* `self` *and* `other`.
    ///
    /// This is equivalent to using the `&` operator (e.g.
    /// [`ops::BitAnd`]), as in `flags & other`.
    ///
    /// [`ops::BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }

    /// Returns the union of between the flags in `self` and `other`.
    ///
    /// Specifically, the returned set contains all flags which are
    /// present in *either* `self` *or* `other`, including any which are
    /// present in both.
    ///
    /// This is equivalent to using the `|` operator (e.g.
    /// [`ops::BitOr`]), as in `flags | other`.
    ///
    /// [`ops::BitOr`]: https://doc.rust-lang.org/std/ops/trait.BitOr.html
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// Returns the difference between the flags in `self` and `other`.
    ///
    /// Specifically, the returned set contains all flags present in
    /// `self`, except for the ones present in `other`.
    ///
    /// It is also conceptually equivalent to the "bit-clear" operation:
    /// `flags & !other` (and this syntax is also supported).
    ///
    /// This is equivalent to using the `-` operator (e.g.
    /// [`ops::Sub`]), as in `flags - other`.
    ///
    /// [`ops::Sub`]: https://doc.rust-lang.org/std/ops/trait.Sub.html
    #[inline]
    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }
}

impl std::ops::BitOr for CompositeGlyphFlags {
    type Output = Self;

    /// Returns the union of the two sets of flags.
    #[inline]
    fn bitor(self, other: CompositeGlyphFlags) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
}

impl std::ops::BitOrAssign for CompositeGlyphFlags {
    /// Adds the set of flags.
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.bits |= other.bits;
    }
}

impl std::ops::BitXor for CompositeGlyphFlags {
    type Output = Self;

    /// Returns the left flags, but with all the right flags toggled.
    #[inline]
    fn bitxor(self, other: Self) -> Self {
        Self {
            bits: self.bits ^ other.bits,
        }
    }
}

impl std::ops::BitXorAssign for CompositeGlyphFlags {
    /// Toggles the set of flags.
    #[inline]
    fn bitxor_assign(&mut self, other: Self) {
        self.bits ^= other.bits;
    }
}

impl std::ops::BitAnd for CompositeGlyphFlags {
    type Output = Self;

    /// Returns the intersection between the two sets of flags.
    #[inline]
    fn bitand(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }
}

impl std::ops::BitAndAssign for CompositeGlyphFlags {
    /// Disables all flags disabled in the set.
    #[inline]
    fn bitand_assign(&mut self, other: Self) {
        self.bits &= other.bits;
    }
}

impl std::ops::Sub for CompositeGlyphFlags {
    type Output = Self;

    /// Returns the set difference of the two sets of flags.
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }
}

impl std::ops::SubAssign for CompositeGlyphFlags {
    /// Disables all flags enabled in the set.
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.bits &= !other.bits;
    }
}

impl std::ops::Not for CompositeGlyphFlags {
    type Output = Self;

    /// Returns the complement of this set of flags.
    #[inline]
    fn not(self) -> Self {
        Self { bits: !self.bits } & Self::all()
    }
}

impl std::fmt::Debug for CompositeGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let members: &[(&str, Self)] = &[
            ("ARG_1_AND_2_ARE_WORDS", Self::ARG_1_AND_2_ARE_WORDS),
            ("ARGS_ARE_XY_VALUES", Self::ARGS_ARE_XY_VALUES),
            ("ROUND_XY_TO_GRID", Self::ROUND_XY_TO_GRID),
            ("WE_HAVE_A_SCALE", Self::WE_HAVE_A_SCALE),
            ("MORE_COMPONENTS", Self::MORE_COMPONENTS),
            ("WE_HAVE_AN_X_AND_Y_SCALE", Self::WE_HAVE_AN_X_AND_Y_SCALE),
            ("WE_HAVE_A_TWO_BY_TWO", Self::WE_HAVE_A_TWO_BY_TWO),
            ("WE_HAVE_INSTRUCTIONS", Self::WE_HAVE_INSTRUCTIONS),
            ("USE_MY_METRICS", Self::USE_MY_METRICS),
            ("OVERLAP_COMPOUND", Self::OVERLAP_COMPOUND),
            ("SCALED_COMPONENT_OFFSET", Self::SCALED_COMPONENT_OFFSET),
            ("UNSCALED_COMPONENT_OFFSET", Self::UNSCALED_COMPONENT_OFFSET),
        ];
        let mut first = true;
        for (name, value) in members {
            if self.contains(*value) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str(name)?;
            }
        }
        if first {
            f.write_str("(empty)")?;
        }
        Ok(())
    }
}

impl std::fmt::Binary for CompositeGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Binary::fmt(&self.bits, f)
    }
}

impl std::fmt::Octal for CompositeGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Octal::fmt(&self.bits, f)
    }
}

impl std::fmt::LowerHex for CompositeGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::LowerHex::fmt(&self.bits, f)
    }
}

impl std::fmt::UpperHex for CompositeGlyphFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.bits, f)
    }
}

impl font_types::Scalar for CompositeGlyphFlags {
    type Raw = <u16 as font_types::Scalar>::Raw;
    fn to_raw(self) -> Self::Raw {
        self.bits().to_raw()
    }
    fn from_raw(raw: Self::Raw) -> Self {
        let t = <u16>::from_raw(raw);
        Self::from_bits_truncate(t)
    }
}

#[cfg(feature = "traversal")]
impl<'a> From<CompositeGlyphFlags> for FieldType<'a> {
    fn from(src: CompositeGlyphFlags) -> FieldType<'a> {
        src.bits().into()
    }
}

/// Simple or composite glyph.
#[derive(Clone)]
pub enum Glyph<'a> {
    Simple(SimpleGlyph<'a>),
    Composite(CompositeGlyph<'a>),
}

impl<'a> Glyph<'a> {
    /// If the number of contours is greater than or equal to zero,
    /// this is a simple glyph. If negative, this is a composite glyph
    /// — the value -1 should be used for composite glyphs.
    pub fn number_of_contours(&self) -> i16 {
        match self {
            Self::Simple(item) => item.number_of_contours(),
            Self::Composite(item) => item.number_of_contours(),
        }
    }

    /// Minimum x for coordinate data.
    pub fn x_min(&self) -> i16 {
        match self {
            Self::Simple(item) => item.x_min(),
            Self::Composite(item) => item.x_min(),
        }
    }

    /// Minimum y for coordinate data.
    pub fn y_min(&self) -> i16 {
        match self {
            Self::Simple(item) => item.y_min(),
            Self::Composite(item) => item.y_min(),
        }
    }

    /// Maximum x for coordinate data.
    pub fn x_max(&self) -> i16 {
        match self {
            Self::Simple(item) => item.x_max(),
            Self::Composite(item) => item.x_max(),
        }
    }

    /// Maximum y for coordinate data.
    pub fn y_max(&self) -> i16 {
        match self {
            Self::Simple(item) => item.y_max(),
            Self::Composite(item) => item.y_max(),
        }
    }
}

impl<'a> FontRead<'a> for Glyph<'a> {
    fn read(data: FontData<'a>) -> Result<Self, ReadError> {
        let format: i16 = data.read_at(0usize)?;

        #[allow(clippy::redundant_guards)]
        match format {
            format if format >= 0 => Ok(Self::Simple(FontRead::read(data)?)),
            format if format < 0 => Ok(Self::Composite(FontRead::read(data)?)),
            other => Err(ReadError::InvalidFormat(other.into())),
        }
    }
}

#[cfg(feature = "traversal")]
impl<'a> Glyph<'a> {
    fn dyn_inner<'b>(&'b self) -> &'b dyn SomeTable<'a> {
        match self {
            Self::Simple(table) => table,
            Self::Composite(table) => table,
        }
    }
}

#[cfg(feature = "traversal")]
impl<'a> std::fmt::Debug for Glyph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.dyn_inner().fmt(f)
    }
}

#[cfg(feature = "traversal")]
impl<'a> SomeTable<'a> for Glyph<'a> {
    fn type_name(&self) -> &str {
        self.dyn_inner().type_name()
    }
    fn get_field(&self, idx: usize) -> Option<Field<'a>> {
        self.dyn_inner().get_field(idx)
    }
}
