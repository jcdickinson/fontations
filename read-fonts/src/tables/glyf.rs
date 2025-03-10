//! The [glyf (Glyph Data)](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) table

pub mod bytecode;

use bytemuck::AnyBitPattern;
use core::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};
use std::fmt;
use types::{F26Dot6, Pen, Point};

include!("../../generated/generated_glyf.rs");

/// Marker bits for point flags that are set during variation delta
/// processing and hinting.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct PointMarker(u8);

impl PointMarker {
    /// Marker for points that have an explicit delta in a glyph variation
    /// tuple.
    pub const HAS_DELTA: Self = Self(0x4);

    /// Marker that signifies that the x coordinate of a point has been touched
    /// by an IUP hinting instruction.
    pub const TOUCHED_X: Self = Self(0x10);

    /// Marker that signifies that the y coordinate of a point has been touched
    /// by an IUP hinting instruction.
    pub const TOUCHED_Y: Self = Self(0x20);

    /// Marker that signifies that the both coordinates of a point has been touched
    /// by an IUP hinting instruction.
    pub const TOUCHED: Self = Self(Self::TOUCHED_X.0 | Self::TOUCHED_Y.0);
}

/// Flags describing the properties of a point.
///
/// Some properties, such as on- and off-curve flags are intrinsic to the point
/// itself. Others, designated as markers are set and cleared while an outline
/// is being transformed during variation application and hinting.
#[derive(
    Copy, Clone, PartialEq, Eq, Default, Debug, bytemuck::AnyBitPattern, bytemuck::NoUninit,
)]
#[repr(transparent)]
pub struct PointFlags(u8);

impl PointFlags {
    // Note: OFF_CURVE_QUAD is signified by the absence of both ON_CURVE
    // and OFF_CURVE_CUBIC bits, per FreeType and TrueType convention.
    const ON_CURVE: u8 = SimpleGlyphFlags::ON_CURVE_POINT.bits;
    const OFF_CURVE_CUBIC: u8 = SimpleGlyphFlags::CUBIC.bits;
    const CURVE_MASK: u8 = Self::ON_CURVE | Self::OFF_CURVE_CUBIC;

    /// Creates a new on curve point flag.
    pub fn on_curve() -> Self {
        Self(Self::ON_CURVE)
    }

    /// Creates a new off curve quadratic point flag.
    pub fn off_curve_quad() -> Self {
        Self(0)
    }

    /// Creates a new off curve cubic point flag.
    pub fn off_curve_cubic() -> Self {
        Self(Self::OFF_CURVE_CUBIC)
    }

    /// Creates a point flag from the given bits. These are truncated
    /// to ignore markers.
    pub fn from_bits(bits: u8) -> Self {
        Self(bits & Self::CURVE_MASK)
    }

    /// Returns true if this is an on curve point.
    pub fn is_on_curve(self) -> bool {
        self.0 & Self::ON_CURVE != 0
    }

    /// Returns true if this is an off curve quadratic point.
    pub fn is_off_curve_quad(self) -> bool {
        self.0 & Self::CURVE_MASK == 0
    }

    /// Returns true if this is an off curve cubic point.
    pub fn is_off_curve_cubic(self) -> bool {
        self.0 & Self::OFF_CURVE_CUBIC != 0
    }

    pub fn is_off_curve(self) -> bool {
        self.is_off_curve_quad() || self.is_off_curve_cubic()
    }

    /// Flips the state of the on curve flag.
    ///
    /// This is used for the TrueType `FLIPPT` instruction.
    pub fn flip_on_curve(&mut self) {
        self.0 ^= 1;
    }

    /// Enables the on curve flag.
    ///
    /// This is used for the TrueType `FLIPRGON` instruction.
    pub fn set_on_curve(&mut self) {
        self.0 |= Self::ON_CURVE;
    }

    /// Disables the on curve flag.
    ///
    /// This is used for the TrueType `FLIPRGOFF` instruction.
    pub fn clear_on_curve(&mut self) {
        self.0 &= !Self::ON_CURVE;
    }

    /// Returns true if the given marker is set for this point.
    pub fn has_marker(self, marker: PointMarker) -> bool {
        self.0 & marker.0 != 0
    }

    /// Applies the given marker to this point.
    pub fn set_marker(&mut self, marker: PointMarker) {
        self.0 |= marker.0;
    }

    /// Clears the given marker for this point.
    pub fn clear_marker(&mut self, marker: PointMarker) {
        self.0 &= !marker.0
    }
}

/// Trait for types that are usable for TrueType point coordinates.
pub trait PointCoord:
    Copy
    + Default
    // You could bytemuck with me
    + AnyBitPattern
    // You could compare me
    + PartialEq
    + PartialOrd
    // You could do math with me
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + Div<Output = Self>
    + Mul<Output = Self>
    + MulAssign {
    fn from_fixed(x: Fixed) -> Self;
    fn from_i32(x: i32) -> Self;
    fn to_f32(self) -> f32;
    fn midpoint(self, other: Self) -> Self;
}

impl<'a> SimpleGlyph<'a> {
    /// Returns the total number of points.
    pub fn num_points(&self) -> usize {
        self.end_pts_of_contours()
            .last()
            .map(|last| last.get() as usize + 1)
            .unwrap_or(0)
    }

    /// Returns true if the contours in the simple glyph may overlap.
    pub fn has_overlapping_contours(&self) -> bool {
        // Checks the first flag for the OVERLAP_SIMPLE bit.
        // Spec says: "When used, it must be set on the first flag byte for
        // the glyph."
        FontData::new(self.glyph_data())
            .read_at::<SimpleGlyphFlags>(0)
            .map(|flag| flag.contains(SimpleGlyphFlags::OVERLAP_SIMPLE))
            .unwrap_or_default()
    }

    /// Reads points and flags into the provided buffers.
    ///
    /// Drops all flag bits except on-curve. The lengths of the buffers must be
    /// equal to the value returned by [num_points](Self::num_points).
    ///
    /// ## Performance
    ///
    /// As the name implies, this is faster than using the iterator returned by
    /// [points](Self::points) so should be used when it is possible to
    /// preallocate buffers.
    pub fn read_points_fast<C: PointCoord>(
        &self,
        points: &mut [Point<C>],
        flags: &mut [PointFlags],
    ) -> Result<(), ReadError> {
        let n_points = self.num_points();
        if points.len() != n_points || flags.len() != n_points {
            return Err(ReadError::InvalidArrayLen);
        }
        let mut cursor = FontData::new(self.glyph_data()).cursor();
        let mut i = 0;
        while i < n_points {
            let flag = cursor.read::<SimpleGlyphFlags>()?;
            let flag_bits = flag.bits();
            if flag.contains(SimpleGlyphFlags::REPEAT_FLAG) {
                let count = (cursor.read::<u8>()? as usize + 1).min(n_points - i);
                for f in &mut flags[i..i + count] {
                    f.0 = flag_bits;
                }
                i += count;
            } else {
                flags[i].0 = flag_bits;
                i += 1;
            }
        }
        let mut x = 0i32;
        for (&point_flags, point) in flags.iter().zip(points.as_mut()) {
            let mut delta = 0i32;
            let flag = SimpleGlyphFlags::from_bits_truncate(point_flags.0);
            if flag.contains(SimpleGlyphFlags::X_SHORT_VECTOR) {
                delta = cursor.read::<u8>()? as i32;
                if !flag.contains(SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR) {
                    delta = -delta;
                }
            } else if !flag.contains(SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR) {
                delta = cursor.read::<i16>()? as i32;
            }
            x = x.wrapping_add(delta);
            point.x = C::from_i32(x);
        }
        let mut y = 0i32;
        for (point_flags, point) in flags.iter_mut().zip(points.as_mut()) {
            let mut delta = 0i32;
            let flag = SimpleGlyphFlags::from_bits_truncate(point_flags.0);
            if flag.contains(SimpleGlyphFlags::Y_SHORT_VECTOR) {
                delta = cursor.read::<u8>()? as i32;
                if !flag.contains(SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR) {
                    delta = -delta;
                }
            } else if !flag.contains(SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR) {
                delta = cursor.read::<i16>()? as i32;
            }
            y = y.wrapping_add(delta);
            point.y = C::from_i32(y);
            *point_flags = PointFlags::from_bits(point_flags.0);
        }
        Ok(())
    }

    /// Returns an iterator over the points in the glyph.
    ///
    /// ## Performance
    ///
    /// This is slower than [read_points_fast](Self::read_points_fast) but
    /// provides access to the points without requiring a preallocated buffer.
    pub fn points(&self) -> impl Iterator<Item = CurvePoint> + 'a + Clone {
        self.points_impl()
            .unwrap_or_else(|| PointIter::new(&[], &[], &[]))
    }

    fn points_impl(&self) -> Option<PointIter<'a>> {
        let end_points = self.end_pts_of_contours();
        let n_points = end_points.last()?.get().checked_add(1)?;
        let data = self.glyph_data();
        let lens = resolve_coords_len(data, n_points).ok()?;
        let total_len = lens.flags + lens.x_coords + lens.y_coords;
        if data.len() < total_len as usize {
            return None;
        }

        let (flags, data) = data.split_at(lens.flags as usize);
        let (x_coords, y_coords) = data.split_at(lens.x_coords as usize);

        Some(PointIter::new(flags, x_coords, y_coords))
    }
}

/// Point with an associated on-curve flag in a simple glyph.
///
/// This type is a simpler representation of the data in the blob.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurvePoint {
    /// X cooordinate.
    pub x: i16,
    /// Y cooordinate.
    pub y: i16,
    /// True if this is an on-curve point.
    pub on_curve: bool,
}

impl CurvePoint {
    /// Construct a new `CurvePoint`
    pub fn new(x: i16, y: i16, on_curve: bool) -> Self {
        Self { x, y, on_curve }
    }

    /// Convenience method to construct an on-curve point
    pub fn on_curve(x: i16, y: i16) -> Self {
        Self::new(x, y, true)
    }

    /// Convenience method to construct an off-curve point
    pub fn off_curve(x: i16, y: i16) -> Self {
        Self::new(x, y, false)
    }
}

#[derive(Clone)]
struct PointIter<'a> {
    flags: Cursor<'a>,
    x_coords: Cursor<'a>,
    y_coords: Cursor<'a>,
    flag_repeats: u8,
    cur_flags: SimpleGlyphFlags,
    cur_x: i16,
    cur_y: i16,
}

impl<'a> Iterator for PointIter<'a> {
    type Item = CurvePoint;
    fn next(&mut self) -> Option<Self::Item> {
        self.advance_flags()?;
        self.advance_points();
        let is_on_curve = self.cur_flags.contains(SimpleGlyphFlags::ON_CURVE_POINT);
        Some(CurvePoint::new(self.cur_x, self.cur_y, is_on_curve))
    }
}

impl<'a> PointIter<'a> {
    fn new(flags: &'a [u8], x_coords: &'a [u8], y_coords: &'a [u8]) -> Self {
        Self {
            flags: FontData::new(flags).cursor(),
            x_coords: FontData::new(x_coords).cursor(),
            y_coords: FontData::new(y_coords).cursor(),
            flag_repeats: 0,
            cur_flags: SimpleGlyphFlags::empty(),
            cur_x: 0,
            cur_y: 0,
        }
    }

    fn advance_flags(&mut self) -> Option<()> {
        if self.flag_repeats == 0 {
            self.cur_flags = SimpleGlyphFlags::from_bits_truncate(self.flags.read().ok()?);
            self.flag_repeats = self
                .cur_flags
                .contains(SimpleGlyphFlags::REPEAT_FLAG)
                .then(|| self.flags.read().ok())
                .flatten()
                .unwrap_or(0)
                + 1;
        }
        self.flag_repeats -= 1;
        Some(())
    }

    fn advance_points(&mut self) {
        let x_short = self.cur_flags.contains(SimpleGlyphFlags::X_SHORT_VECTOR);
        let x_same_or_pos = self
            .cur_flags
            .contains(SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR);
        let y_short = self.cur_flags.contains(SimpleGlyphFlags::Y_SHORT_VECTOR);
        let y_same_or_pos = self
            .cur_flags
            .contains(SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR);

        let delta_x = match (x_short, x_same_or_pos) {
            (true, false) => -(self.x_coords.read::<u8>().unwrap_or(0) as i16),
            (true, true) => self.x_coords.read::<u8>().unwrap_or(0) as i16,
            (false, false) => self.x_coords.read::<i16>().unwrap_or(0),
            _ => 0,
        };

        let delta_y = match (y_short, y_same_or_pos) {
            (true, false) => -(self.y_coords.read::<u8>().unwrap_or(0) as i16),
            (true, true) => self.y_coords.read::<u8>().unwrap_or(0) as i16,
            (false, false) => self.y_coords.read::<i16>().unwrap_or(0),
            _ => 0,
        };

        self.cur_x = self.cur_x.wrapping_add(delta_x);
        self.cur_y = self.cur_y.wrapping_add(delta_y);
    }
}

//taken from ttf_parser https://docs.rs/ttf-parser/latest/src/ttf_parser/tables/glyf.rs.html#1-677
/// Resolves coordinate arrays length.
///
/// The length depends on *Simple Glyph Flags*, so we have to process them all to find it.
fn resolve_coords_len(data: &[u8], points_total: u16) -> Result<FieldLengths, ReadError> {
    let mut cursor = FontData::new(data).cursor();
    let mut flags_left = u32::from(points_total);
    //let mut repeats;
    let mut x_coords_len = 0;
    let mut y_coords_len = 0;
    //let mut flags_seen = 0;
    while flags_left > 0 {
        let flags: SimpleGlyphFlags = cursor.read()?;

        // The number of times a glyph point repeats.
        let repeats = if flags.contains(SimpleGlyphFlags::REPEAT_FLAG) {
            let repeats: u8 = cursor.read()?;
            u32::from(repeats) + 1
        } else {
            1
        };

        if repeats > flags_left {
            return Err(ReadError::MalformedData("repeat count too large in glyf"));
        }

        // Non-obfuscated code below.
        // Branchless version is surprisingly faster.
        //
        // if flags.x_short() {
        //     // Coordinate is 1 byte long.
        //     x_coords_len += repeats;
        // } else if !flags.x_is_same_or_positive_short() {
        //     // Coordinate is 2 bytes long.
        //     x_coords_len += repeats * 2;
        // }
        // if flags.y_short() {
        //     // Coordinate is 1 byte long.
        //     y_coords_len += repeats;
        // } else if !flags.y_is_same_or_positive_short() {
        //     // Coordinate is 2 bytes long.
        //     y_coords_len += repeats * 2;
        // }
        let x_short = SimpleGlyphFlags::X_SHORT_VECTOR;
        let x_long = SimpleGlyphFlags::X_SHORT_VECTOR
            | SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR;
        let y_short = SimpleGlyphFlags::Y_SHORT_VECTOR;
        let y_long = SimpleGlyphFlags::Y_SHORT_VECTOR
            | SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR;
        x_coords_len += ((flags & x_short).bits() != 0) as u32 * repeats;
        x_coords_len += ((flags & x_long).bits() == 0) as u32 * repeats * 2;

        y_coords_len += ((flags & y_short).bits() != 0) as u32 * repeats;
        y_coords_len += ((flags & y_long).bits() == 0) as u32 * repeats * 2;

        flags_left -= repeats;
    }

    Ok(FieldLengths {
        flags: cursor.position()? as u32,
        x_coords: x_coords_len,
        y_coords: y_coords_len,
    })
    //Some((flags_len, x_coords_len, y_coords_len))
}

struct FieldLengths {
    flags: u32,
    x_coords: u32,
    y_coords: u32,
}

/// Transform for a composite component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transform {
    /// X scale factor.
    pub xx: F2Dot14,
    /// YX skew factor.
    pub yx: F2Dot14,
    /// XY skew factor.
    pub xy: F2Dot14,
    /// Y scale factor.
    pub yy: F2Dot14,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            xx: F2Dot14::from_f32(1.0),
            yx: F2Dot14::from_f32(0.0),
            xy: F2Dot14::from_f32(0.0),
            yy: F2Dot14::from_f32(1.0),
        }
    }
}

/// A reference to another glyph. Part of [CompositeGlyph].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    /// Component flags.
    pub flags: CompositeGlyphFlags,
    /// Glyph identifier.
    pub glyph: GlyphId16,
    /// Anchor for component placement.
    pub anchor: Anchor,
    /// Component transformation matrix.
    pub transform: Transform,
}

/// Anchor position for a composite component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Offset { x: i16, y: i16 },
    Point { base: u16, component: u16 },
}

impl<'a> CompositeGlyph<'a> {
    /// Returns an iterator over the components of the composite glyph.
    pub fn components(&self) -> impl Iterator<Item = Component> + 'a + Clone {
        ComponentIter {
            cur_flags: CompositeGlyphFlags::empty(),
            done: false,
            cursor: FontData::new(self.component_data()).cursor(),
        }
    }

    /// Returns an iterator that yields the glyph identifier and flags of each
    /// component in the composite glyph.
    pub fn component_glyphs_and_flags(
        &self,
    ) -> impl Iterator<Item = (GlyphId16, CompositeGlyphFlags)> + 'a + Clone {
        ComponentGlyphIdFlagsIter {
            cur_flags: CompositeGlyphFlags::empty(),
            done: false,
            cursor: FontData::new(self.component_data()).cursor(),
        }
    }

    /// Returns the component count and TrueType interpreter instructions
    /// in a single pass.
    pub fn count_and_instructions(&self) -> (usize, Option<&'a [u8]>) {
        let mut iter = ComponentGlyphIdFlagsIter {
            cur_flags: CompositeGlyphFlags::empty(),
            done: false,
            cursor: FontData::new(self.component_data()).cursor(),
        };
        let mut count = 0;
        while iter.by_ref().next().is_some() {
            count += 1;
        }
        let instructions = if iter
            .cur_flags
            .contains(CompositeGlyphFlags::WE_HAVE_INSTRUCTIONS)
        {
            iter.cursor
                .read::<u16>()
                .ok()
                .map(|len| len as usize)
                .and_then(|len| iter.cursor.read_array(len).ok())
        } else {
            None
        };
        (count, instructions)
    }

    /// Returns the TrueType interpreter instructions.
    pub fn instructions(&self) -> Option<&'a [u8]> {
        self.count_and_instructions().1
    }
}

#[derive(Clone)]
struct ComponentIter<'a> {
    cur_flags: CompositeGlyphFlags,
    done: bool,
    cursor: Cursor<'a>,
}

impl Iterator for ComponentIter<'_> {
    type Item = Component;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let flags: CompositeGlyphFlags = self.cursor.read().ok()?;
        self.cur_flags = flags;
        let glyph = self.cursor.read::<GlyphId16>().ok()?;
        let args_are_words = flags.contains(CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS);
        let args_are_xy_values = flags.contains(CompositeGlyphFlags::ARGS_ARE_XY_VALUES);
        let anchor = match (args_are_xy_values, args_are_words) {
            (true, true) => Anchor::Offset {
                x: self.cursor.read().ok()?,
                y: self.cursor.read().ok()?,
            },
            (true, false) => Anchor::Offset {
                x: self.cursor.read::<i8>().ok()? as _,
                y: self.cursor.read::<i8>().ok()? as _,
            },
            (false, true) => Anchor::Point {
                base: self.cursor.read().ok()?,
                component: self.cursor.read().ok()?,
            },
            (false, false) => Anchor::Point {
                base: self.cursor.read::<u8>().ok()? as _,
                component: self.cursor.read::<u8>().ok()? as _,
            },
        };
        let mut transform = Transform::default();
        if flags.contains(CompositeGlyphFlags::WE_HAVE_A_SCALE) {
            transform.xx = self.cursor.read().ok()?;
            transform.yy = transform.xx;
        } else if flags.contains(CompositeGlyphFlags::WE_HAVE_AN_X_AND_Y_SCALE) {
            transform.xx = self.cursor.read().ok()?;
            transform.yy = self.cursor.read().ok()?;
        } else if flags.contains(CompositeGlyphFlags::WE_HAVE_A_TWO_BY_TWO) {
            transform.xx = self.cursor.read().ok()?;
            transform.yx = self.cursor.read().ok()?;
            transform.xy = self.cursor.read().ok()?;
            transform.yy = self.cursor.read().ok()?;
        }
        self.done = !flags.contains(CompositeGlyphFlags::MORE_COMPONENTS);

        Some(Component {
            flags,
            glyph,
            anchor,
            transform,
        })
    }
}

/// Iterator that only returns glyph identifiers and flags for each component.
///
/// Significantly faster in cases where we're just processing the glyph
/// tree, counting components or accessing instructions.
#[derive(Clone)]
struct ComponentGlyphIdFlagsIter<'a> {
    cur_flags: CompositeGlyphFlags,
    done: bool,
    cursor: Cursor<'a>,
}

impl Iterator for ComponentGlyphIdFlagsIter<'_> {
    type Item = (GlyphId16, CompositeGlyphFlags);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let flags: CompositeGlyphFlags = self.cursor.read().ok()?;
        self.cur_flags = flags;
        let glyph = self.cursor.read::<GlyphId16>().ok()?;
        let args_are_words = flags.contains(CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS);
        if args_are_words {
            self.cursor.advance_by(4);
        } else {
            self.cursor.advance_by(2);
        }
        if flags.contains(CompositeGlyphFlags::WE_HAVE_A_SCALE) {
            self.cursor.advance_by(2);
        } else if flags.contains(CompositeGlyphFlags::WE_HAVE_AN_X_AND_Y_SCALE) {
            self.cursor.advance_by(4);
        } else if flags.contains(CompositeGlyphFlags::WE_HAVE_A_TWO_BY_TWO) {
            self.cursor.advance_by(8);
        }
        self.done = !flags.contains(CompositeGlyphFlags::MORE_COMPONENTS);
        Some((glyph, flags))
    }
}

#[cfg(feature = "traversal")]
impl<'a> SomeTable<'a> for Component {
    fn type_name(&self) -> &str {
        "Component"
    }

    fn get_field(&self, idx: usize) -> Option<Field<'a>> {
        match idx {
            0 => Some(Field::new("flags", self.flags.bits())),
            1 => Some(Field::new("glyph", self.glyph)),
            2 => match self.anchor {
                Anchor::Point { base, .. } => Some(Field::new("base", base)),
                Anchor::Offset { x, .. } => Some(Field::new("x", x)),
            },
            3 => match self.anchor {
                Anchor::Point { component, .. } => Some(Field::new("component", component)),
                Anchor::Offset { y, .. } => Some(Field::new("y", y)),
            },
            _ => None,
        }
    }
}

/// Errors that can occur when converting an outline to a path.
#[derive(Clone, Debug)]
pub enum ToPathError {
    /// Contour end point at this index was less than its preceding end point.
    ContourOrder(usize),
    /// Expected a quadratic off-curve point at this index.
    ExpectedQuad(usize),
    /// Expected a quadratic off-curve or on-curve point at this index.
    ExpectedQuadOrOnCurve(usize),
    /// Expected a cubic off-curve point at this index.
    ExpectedCubic(usize),
    /// Expected number of points to == number of flags
    PointFlagMismatch { num_points: usize, num_flags: usize },
}

impl fmt::Display for ToPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ContourOrder(ix) => write!(
                f,
                "Contour end point at index {ix} was less than preceeding end point"
            ),
            Self::ExpectedQuad(ix) => write!(f, "Expected quadatic off-curve point at index {ix}"),
            Self::ExpectedQuadOrOnCurve(ix) => write!(
                f,
                "Expected quadatic off-curve or on-curve point at index {ix}"
            ),
            Self::ExpectedCubic(ix) => write!(f, "Expected cubic off-curve point at index {ix}"),
            Self::PointFlagMismatch {
                num_points,
                num_flags,
            } => write!(
                f,
                "Number of points ({num_points}) and flags ({num_flags}) must match"
            ),
        }
    }
}

/// The order to process points in a glyf point stream is ambiguous when the first point is
/// off-curve. Major implementations differ. Which one would you like to match?
///
/// **If you add a new one make sure to update the fuzzer**
#[derive(Debug, Default, Copy, Clone)]
pub enum ToPathStyle {
    /// If the first point is off-curve, check if the last is on-curve
    /// If it is, start there. If it isn't, start at the implied midpoint between first and last.
    #[default]
    FreeType,
    /// If the first point is off-curve, check if the second is on-curve.
    /// If it is, start there. If it isn't, start at the implied midpoint between first and second.
    ///
    /// Matches hb-draw's interpretation of a pointstream.
    HarfBuzz,
}

#[derive(Debug, Copy, Clone)]
enum OnCurve {
    Implicit,
    Explicit,
}

impl OnCurve {
    #[inline(always)]
    fn endpoint<C: PointCoord>(&self, last_offcurve: Point<C>, current: Point<C>) -> Point<C> {
        match self {
            OnCurve::Implicit => midpoint(last_offcurve, current),
            OnCurve::Explicit => current,
        }
    }
}

// FreeType uses integer division to compute midpoints.
// See: https://github.com/freetype/freetype/blob/de8b92dd7ec634e9e2b25ef534c54a3537555c11/src/base/ftoutln.c#L123
#[inline(always)]
fn midpoint<C: PointCoord>(a: Point<C>, b: Point<C>) -> Point<C> {
    Point::new(a.x.midpoint(b.x), a.y.midpoint(b.y))
}

#[derive(Debug)]
struct Contour<'a, C>
where
    C: PointCoord,
{
    /// The offset into the glyphs point stream our points start.
    ///
    /// Allows us to report error indices that are correct relative to the containing point stream.
    base_offset: usize,
    /// The points for this contour, sliced from a glyphs point stream
    points: &'a [Point<C>],
    /// The flags for points, sliced from a glyphs flag stream
    flags: &'a [PointFlags],
    path_style: ToPathStyle,
}

#[derive(Debug)]
struct ContourIter<'a, C>
where
    C: PointCoord,
{
    contour: &'a Contour<'a, C>,
    /// Wrapping index into points/flags of contour.
    ix: usize,
    /// Number of points yet to be processed
    points_pending: usize,
    first_move: Point<C>,
    /// Indices of off-curve points yet to be processed
    off_curve: [usize; 2],
    /// Number of live entries in off_curve
    off_curve_pending: usize,
}

impl<'a, C> ContourIter<'a, C>
where
    C: PointCoord,
{
    fn new(contour: &'a Contour<'_, C>) -> Result<Self, ToPathError> {
        // By default start at [0], unless [0] is off-curve
        let mut start_ix = 0;
        let mut num_pending = contour.points.len();
        let mut first_move = Default::default();
        if !contour.points.is_empty() {
            if contour.flags[0].is_off_curve_cubic() {
                return Err(ToPathError::ExpectedQuadOrOnCurve(contour.base_offset));
            }

            // Establish our first move and the starting index for our waltz over the points
            first_move = contour.points[0];
            if contour.flags[0].is_off_curve_quad() {
                let maybe_oncurve_ix = match contour.path_style {
                    ToPathStyle::FreeType => contour.points.len() - 1,
                    ToPathStyle::HarfBuzz => (start_ix + 1).min(contour.points.len() - 1),
                };
                if contour.flags[maybe_oncurve_ix].is_on_curve() {
                    start_ix = maybe_oncurve_ix + 1;
                    first_move = contour.points[maybe_oncurve_ix];
                    num_pending = num_pending.saturating_sub(1); // first move consumes the first point
                } else {
                    // where we looked for an on-curve was also off. Take the implicit oncurve in between as first move.
                    first_move = midpoint(contour.points[0], contour.points[maybe_oncurve_ix]);
                    start_ix = match contour.path_style {
                        ToPathStyle::FreeType => 0,
                        ToPathStyle::HarfBuzz => 1,
                    };
                }
            } else {
                start_ix = 1;
                num_pending = num_pending.saturating_sub(1); // first move consumes the first point
            }
        }
        Ok(Self {
            contour,
            ix: start_ix,
            points_pending: num_pending,
            first_move,
            off_curve: [0, 0],
            off_curve_pending: 0,
        })
    }

    fn push_off_curve(&mut self, ix: usize) {
        self.off_curve[self.off_curve_pending] = ix;
        self.off_curve_pending += 1;
    }

    fn quad_to(
        &mut self,
        curr: Point<C>,
        oncurve: OnCurve,
        pen: &mut impl Pen,
    ) -> Result<(), ToPathError> {
        let buf_ix = self.off_curve[0];

        if !self.contour.flags[buf_ix].is_off_curve_quad() {
            return Err(ToPathError::ExpectedQuad(self.contour.base_offset + buf_ix));
        }
        let c0 = self.contour.points[buf_ix];
        let end = oncurve.endpoint(c0, curr).map(C::to_f32);
        let c0 = c0.map(C::to_f32);
        pen.quad_to(c0.x, c0.y, end.x, end.y);

        self.off_curve_pending = 0;
        Ok(())
    }

    fn cubic_to(
        &mut self,
        curr: Point<C>,
        oncurve: OnCurve,
        pen: &mut impl Pen,
    ) -> Result<(), ToPathError> {
        let buf_ix_0 = self.off_curve[0];
        let buf_ix_1 = self.off_curve[1];

        if !self.contour.flags[buf_ix_0].is_off_curve_cubic() {
            return Err(ToPathError::ExpectedCubic(
                self.contour.base_offset + buf_ix_0,
            ));
        }
        if !self.contour.flags[buf_ix_1].is_off_curve_cubic() {
            return Err(ToPathError::ExpectedCubic(
                self.contour.base_offset + buf_ix_1,
            ));
        }

        let c0 = self.contour.points[buf_ix_0].map(C::to_f32);
        let c1 = self.contour.points[buf_ix_1];
        let end = oncurve.endpoint(c1, curr).map(C::to_f32);
        let c1 = c1.map(C::to_f32);
        pen.curve_to(c0.x, c0.y, c1.x, c1.y, end.x, end.y);

        self.off_curve_pending = 0;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.points_pending == 0 && self.off_curve_pending == 0
    }

    fn next(&mut self, pen: &mut impl Pen) -> Result<(), ToPathError> {
        if self.is_empty() {
            return Ok(());
        }

        if self.points_pending > 0 {
            // Take the next point in the stream
            // Wrapping read of flags and points
            let ix = self.ix % self.contour.points.len();
            self.points_pending -= 1;
            self.ix += 1;
            let (flag, curr) = (self.contour.flags[ix], self.contour.points[ix]);
            if flag.is_off_curve_quad() {
                // quads can have 0 or 1 buffered point. If there is one draw a quad to the implicit oncurve.
                if self.off_curve_pending > 0 {
                    self.quad_to(curr, OnCurve::Implicit, pen)?;
                }
                self.push_off_curve(ix);
            } else if flag.is_off_curve_cubic() {
                // If this is the third consecutive cubic off-curve there's an implicit oncurve between the last two
                if self.off_curve_pending > 1 {
                    self.cubic_to(curr, OnCurve::Implicit, pen)?;
                }
                self.push_off_curve(ix);
            } else if flag.is_on_curve() {
                // A real oncurve! What to draw depends on how many off curves are pending.
                match self.off_curve_pending {
                    2 => self.cubic_to(curr, OnCurve::Explicit, pen)?,
                    1 => self.quad_to(curr, OnCurve::Explicit, pen)?,
                    _ => {
                        // only zero should match and that's a line
                        debug_assert!(self.off_curve_pending == 0);
                        let curr = curr.map(C::to_f32);
                        pen.line_to(curr.x, curr.y);
                    }
                }
            }
        } else {
            // We weren't empty when we entered and we have no moe points so we must have pending off-curves
            match self.off_curve_pending {
                2 => self.cubic_to(self.first_move, OnCurve::Explicit, pen)?,
                1 => self.quad_to(self.first_move, OnCurve::Explicit, pen)?,
                _ => debug_assert!(
                    false,
                    "We're in a weird state. {} pending, {} off-curve.",
                    self.points_pending, self.off_curve_pending
                ),
            }
        }
        Ok(())
    }
}

impl<'a, C> Contour<'a, C>
where
    C: PointCoord,
{
    fn new(
        base_offset: usize,
        points: &'a [Point<C>],
        flags: &'a [PointFlags],
        path_style: ToPathStyle,
    ) -> Result<Self, ToPathError> {
        if points.len() != flags.len() {
            return Err(ToPathError::PointFlagMismatch {
                num_points: points.len(),
                num_flags: flags.len(),
            });
        }
        Ok(Self {
            base_offset,
            points,
            flags,
            path_style,
        })
    }

    fn draw(self, pen: &mut impl Pen) -> Result<(), ToPathError> {
        let mut it = ContourIter::new(&self)?;
        if it.is_empty() {
            return Ok(());
        }

        let first_move = it.first_move.map(C::to_f32);
        pen.move_to(first_move.x, first_move.y);

        while !it.is_empty() {
            it.next(pen)?;
        }

        // we know there was at least one point so close out the contour
        pen.close();

        Ok(())
    }
}

/// Converts a `glyf` outline described by points, flags and contour end points
/// to a sequence of path elements and invokes the appropriate callback on the
/// given pen for each.
///
/// The input points can have any coordinate type that implements
/// [`PointCoord`]. Output points are always generated in `f32`.
///
/// This is roughly equivalent to [`FT_Outline_Decompose`](https://freetype.org/freetype2/docs/reference/ft2-outline_processing.html#ft_outline_decompose).
pub fn to_path<C: PointCoord>(
    points: &[Point<C>],
    flags: &[PointFlags],
    contours: &[u16],
    path_style: ToPathStyle,
    pen: &mut impl Pen,
) -> Result<(), ToPathError> {
    for contour_ix in 0..contours.len() {
        let start_ix = (contour_ix > 0)
            .then(|| contours[contour_ix - 1] as usize + 1)
            .unwrap_or_default();
        let end_ix = contours[contour_ix] as usize;
        if end_ix < start_ix || end_ix >= points.len() {
            return Err(ToPathError::ContourOrder(contour_ix));
        }
        Contour::new(
            start_ix,
            &points[start_ix..=end_ix],
            &flags[start_ix..=end_ix],
            path_style,
        )?
        .draw(pen)?;
    }
    Ok(())
}

impl Anchor {
    /// Compute the flags that describe this anchor
    pub fn compute_flags(&self) -> CompositeGlyphFlags {
        const I8_RANGE: Range<i16> = i8::MIN as i16..i8::MAX as i16 + 1;
        const U8_MAX: u16 = u8::MAX as u16;

        let mut flags = CompositeGlyphFlags::empty();
        match self {
            Anchor::Offset { x, y } => {
                flags |= CompositeGlyphFlags::ARGS_ARE_XY_VALUES;
                if !I8_RANGE.contains(x) || !I8_RANGE.contains(y) {
                    flags |= CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS;
                }
            }
            Anchor::Point { base, component } => {
                if base > &U8_MAX || component > &U8_MAX {
                    flags |= CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS;
                }
            }
        }
        flags
    }
}

impl Transform {
    /// Compute the flags that describe this transform
    pub fn compute_flags(&self) -> CompositeGlyphFlags {
        if self.yx != F2Dot14::ZERO || self.xy != F2Dot14::ZERO {
            CompositeGlyphFlags::WE_HAVE_A_TWO_BY_TWO
        } else if self.xx != self.yy {
            CompositeGlyphFlags::WE_HAVE_AN_X_AND_Y_SCALE
        } else if self.xx != F2Dot14::ONE {
            CompositeGlyphFlags::WE_HAVE_A_SCALE
        } else {
            CompositeGlyphFlags::empty()
        }
    }
}

impl PointCoord for F26Dot6 {
    fn from_fixed(x: Fixed) -> Self {
        x.to_f26dot6()
    }

    fn from_i32(x: i32) -> Self {
        Self::from_i32(x)
    }

    fn to_f32(self) -> f32 {
        self.to_f32()
    }

    fn midpoint(self, other: Self) -> Self {
        Self::from_bits((self.to_bits() + other.to_bits()) / 2)
    }
}

impl PointCoord for Fixed {
    fn from_fixed(x: Fixed) -> Self {
        x
    }

    fn from_i32(x: i32) -> Self {
        Self::from_i32(x)
    }

    fn to_f32(self) -> f32 {
        self.to_f32()
    }

    fn midpoint(self, other: Self) -> Self {
        Self::from_bits((self.to_bits() + other.to_bits()) / 2)
    }
}

impl PointCoord for i32 {
    fn from_fixed(x: Fixed) -> Self {
        x.to_i32()
    }

    fn from_i32(x: i32) -> Self {
        x
    }

    fn to_f32(self) -> f32 {
        self as f32
    }

    fn midpoint(self, other: Self) -> Self {
        (self + other) / 2
    }
}

impl PointCoord for f32 {
    fn from_fixed(x: Fixed) -> Self {
        x.to_f32()
    }

    fn from_i32(x: i32) -> Self {
        x as f32
    }

    fn to_f32(self) -> f32 {
        self
    }

    fn midpoint(self, other: Self) -> Self {
        // HarfBuzz uses a lerp here so we copy the style to
        // preserve compatibility
        self + 0.5 * (other - self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{FontRef, GlyphId, TableProvider};

    fn assert_all_off_curve_path_to_svg(expected: &str, path_style: ToPathStyle) {
        fn pt(x: i32, y: i32) -> Point<F26Dot6> {
            Point::new(x, y).map(F26Dot6::from_bits)
        }
        let flags = [PointFlags::off_curve_quad(); 4];
        let contours = [3];
        // This test is meant to prevent a bug where the first move-to was computed improperly
        // for a contour consisting of all off curve points.
        // In this case, the start of the path should be the midpoint between the first and last points.
        // For this test case (in 26.6 fixed point): [(640, 128) + (128, 128)] / 2 = (384, 128)
        // which becomes (6.0, 2.0) when converted to floating point.
        let points = [pt(640, 128), pt(256, 64), pt(640, 64), pt(128, 128)];
        struct SvgPen(String);
        impl Pen for SvgPen {
            fn move_to(&mut self, x: f32, y: f32) {
                self.0.push_str(&format!("M{x:.1},{y:.1} "));
            }
            fn line_to(&mut self, x: f32, y: f32) {
                self.0.push_str(&format!("L{x:.1},{y:.1} "));
            }
            fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
                self.0
                    .push_str(&format!("Q{cx0:.1},{cy0:.1} {x:.1},{y:.1} "));
            }
            fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
                self.0.push_str(&format!(
                    "C{cx0:.1},{cy0:.1} {cx1:.1},{cy1:.1} {x:.1},{y:.1} "
                ));
            }
            fn close(&mut self) {
                self.0.push_str("z ");
            }
        }
        let mut pen = SvgPen(String::default());
        to_path(&points, &flags, &contours, path_style, &mut pen).unwrap();
        assert_eq!(pen.0.trim(), expected);
    }

    #[test]
    fn all_off_curve_to_path_offcurve_scan_backward() {
        assert_all_off_curve_path_to_svg(
            "M6.0,2.0 Q10.0,2.0 7.0,1.5 Q4.0,1.0 7.0,1.0 Q10.0,1.0 6.0,1.5 Q2.0,2.0 6.0,2.0 z",
            ToPathStyle::FreeType,
        );
    }

    #[test]
    fn all_off_curve_to_path_offcurve_scan_forward() {
        assert_all_off_curve_path_to_svg(
            "M7.0,1.5 Q4.0,1.0 7.0,1.0 Q10.0,1.0 6.0,1.5 Q2.0,2.0 6.0,2.0 Q10.0,2.0 7.0,1.5 z",
            ToPathStyle::HarfBuzz,
        );
    }

    #[test]
    fn simple_glyph() {
        let font = FontRef::new(font_test_data::COLR_GRADIENT_RECT).unwrap();
        let loca = font.loca(None).unwrap();
        let glyf = font.glyf().unwrap();
        let glyph = loca.get_glyf(GlyphId::new(0), &glyf).unwrap().unwrap();
        assert_eq!(glyph.number_of_contours(), 2);
        let simple_glyph = if let Glyph::Simple(simple) = glyph {
            simple
        } else {
            panic!("expected simple glyph");
        };
        assert_eq!(
            simple_glyph
                .end_pts_of_contours()
                .iter()
                .map(|x| x.get())
                .collect::<Vec<_>>(),
            &[3, 7]
        );
        assert_eq!(
            simple_glyph
                .points()
                .map(|pt| (pt.x, pt.y, pt.on_curve))
                .collect::<Vec<_>>(),
            &[
                (5, 0, true),
                (5, 100, true),
                (45, 100, true),
                (45, 0, true),
                (10, 5, true),
                (40, 5, true),
                (40, 95, true),
                (10, 95, true),
            ]
        );
    }

    // Test helper to enumerate all TrueType glyphs in the given font
    fn all_glyphs(font_data: &[u8]) -> impl Iterator<Item = Option<Glyph>> {
        let font = FontRef::new(font_data).unwrap();
        let loca = font.loca(None).unwrap();
        let glyf = font.glyf().unwrap();
        let glyph_count = font.maxp().unwrap().num_glyphs() as u32;
        (0..glyph_count).map(move |gid| loca.get_glyf(GlyphId::new(gid), &glyf).unwrap())
    }

    #[test]
    fn simple_glyph_overlapping_contour_flag() {
        let gids_with_overlap: Vec<_> = all_glyphs(font_test_data::VAZIRMATN_VAR)
            .enumerate()
            .filter_map(|(gid, glyph)| match glyph {
                Some(Glyph::Simple(glyph)) if glyph.has_overlapping_contours() => Some(gid),
                _ => None,
            })
            .collect();
        // Only GID 3 has the overlap bit set
        let expected_gids_with_overlap = vec![3];
        assert_eq!(expected_gids_with_overlap, gids_with_overlap);
    }

    #[test]
    fn composite_glyph_overlapping_contour_flag() {
        let gids_components_with_overlap: Vec<_> = all_glyphs(font_test_data::VAZIRMATN_VAR)
            .enumerate()
            .filter_map(|(gid, glyph)| match glyph {
                Some(Glyph::Composite(glyph)) => Some((gid, glyph)),
                _ => None,
            })
            .flat_map(|(gid, glyph)| {
                glyph
                    .components()
                    .enumerate()
                    .filter_map(move |(comp_ix, comp)| {
                        comp.flags
                            .contains(CompositeGlyphFlags::OVERLAP_COMPOUND)
                            .then_some((gid, comp_ix))
                    })
            })
            .collect();
        // Only GID 2, component 1 has the overlap bit set
        let expected_gids_components_with_overlap = vec![(2, 1)];
        assert_eq!(
            expected_gids_components_with_overlap,
            gids_components_with_overlap
        );
    }

    #[test]
    fn compute_anchor_flags() {
        let anchor = Anchor::Offset { x: -128, y: 127 };
        assert_eq!(
            anchor.compute_flags(),
            CompositeGlyphFlags::ARGS_ARE_XY_VALUES
        );

        let anchor = Anchor::Offset { x: -129, y: 127 };
        assert_eq!(
            anchor.compute_flags(),
            CompositeGlyphFlags::ARGS_ARE_XY_VALUES | CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS
        );
        let anchor = Anchor::Offset { x: -1, y: 128 };
        assert_eq!(
            anchor.compute_flags(),
            CompositeGlyphFlags::ARGS_ARE_XY_VALUES | CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS
        );

        let anchor = Anchor::Point {
            base: 255,
            component: 20,
        };
        assert_eq!(anchor.compute_flags(), CompositeGlyphFlags::empty());

        let anchor = Anchor::Point {
            base: 256,
            component: 20,
        };
        assert_eq!(
            anchor.compute_flags(),
            CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS
        )
    }

    #[test]
    fn compute_transform_flags() {
        fn make_xform(xx: f32, yx: f32, xy: f32, yy: f32) -> Transform {
            Transform {
                xx: F2Dot14::from_f32(xx),
                yx: F2Dot14::from_f32(yx),
                xy: F2Dot14::from_f32(xy),
                yy: F2Dot14::from_f32(yy),
            }
        }

        assert_eq!(
            make_xform(1.0, 0., 0., 1.0).compute_flags(),
            CompositeGlyphFlags::empty()
        );
        assert_eq!(
            make_xform(2.0, 0., 0., 2.0).compute_flags(),
            CompositeGlyphFlags::WE_HAVE_A_SCALE
        );
        assert_eq!(
            make_xform(2.0, 0., 0., 1.0).compute_flags(),
            CompositeGlyphFlags::WE_HAVE_AN_X_AND_Y_SCALE
        );
        assert_eq!(
            make_xform(2.0, 0., 1.0, 1.0).compute_flags(),
            CompositeGlyphFlags::WE_HAVE_A_TWO_BY_TWO
        );
    }

    #[test]
    fn point_flags_and_marker_bits() {
        let bits = [
            PointFlags::OFF_CURVE_CUBIC,
            PointFlags::ON_CURVE,
            PointMarker::HAS_DELTA.0,
            PointMarker::TOUCHED_X.0,
            PointMarker::TOUCHED_Y.0,
        ];
        // Ensure bits don't overlap
        for (i, a) in bits.iter().enumerate() {
            for b in &bits[i + 1..] {
                assert_eq!(a & b, 0);
            }
        }
    }

    #[test]
    fn cubic_glyf() {
        let font = FontRef::new(font_test_data::CUBIC_GLYF).unwrap();
        let loca = font.loca(None).unwrap();
        let glyf = font.glyf().unwrap();
        let glyph = loca.get_glyf(GlyphId::new(2), &glyf).unwrap().unwrap();
        assert_eq!(glyph.number_of_contours(), 1);
        let simple_glyph = if let Glyph::Simple(simple) = glyph {
            simple
        } else {
            panic!("expected simple glyph");
        };
        assert_eq!(
            simple_glyph
                .points()
                .map(|pt| (pt.x, pt.y, pt.on_curve))
                .collect::<Vec<_>>(),
            &[
                (278, 710, true),
                (278, 470, true),
                (300, 500, false),
                (800, 500, false),
                (998, 470, true),
                (998, 710, true),
            ]
        );
    }
}
