// THIS FILE IS AUTOGENERATED.
// Any changes to this file will be overwritten.
// For more information about how codegen works, see font-codegen/README.md

#[allow(unused_imports)]
use crate::codegen_prelude::*;

/// <https://docs.microsoft.com/en-us/typography/opentype/spec/head>
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct HeadMarker {}

impl HeadMarker {
    fn version_byte_range(&self) -> Range<usize> {
        let start = 0;
        start..start + MajorMinor::RAW_BYTE_LEN
    }
    fn font_revision_byte_range(&self) -> Range<usize> {
        let start = self.version_byte_range().end;
        start..start + Fixed::RAW_BYTE_LEN
    }
    fn checksum_adjustment_byte_range(&self) -> Range<usize> {
        let start = self.font_revision_byte_range().end;
        start..start + u32::RAW_BYTE_LEN
    }
    fn magic_number_byte_range(&self) -> Range<usize> {
        let start = self.checksum_adjustment_byte_range().end;
        start..start + u32::RAW_BYTE_LEN
    }
    fn flags_byte_range(&self) -> Range<usize> {
        let start = self.magic_number_byte_range().end;
        start..start + u16::RAW_BYTE_LEN
    }
    fn units_per_em_byte_range(&self) -> Range<usize> {
        let start = self.flags_byte_range().end;
        start..start + u16::RAW_BYTE_LEN
    }
    fn created_byte_range(&self) -> Range<usize> {
        let start = self.units_per_em_byte_range().end;
        start..start + LongDateTime::RAW_BYTE_LEN
    }
    fn modified_byte_range(&self) -> Range<usize> {
        let start = self.created_byte_range().end;
        start..start + LongDateTime::RAW_BYTE_LEN
    }
    fn x_min_byte_range(&self) -> Range<usize> {
        let start = self.modified_byte_range().end;
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
    fn mac_style_byte_range(&self) -> Range<usize> {
        let start = self.y_max_byte_range().end;
        start..start + u16::RAW_BYTE_LEN
    }
    fn lowest_rec_ppem_byte_range(&self) -> Range<usize> {
        let start = self.mac_style_byte_range().end;
        start..start + u16::RAW_BYTE_LEN
    }
    fn font_direction_hint_byte_range(&self) -> Range<usize> {
        let start = self.lowest_rec_ppem_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn index_to_loc_format_byte_range(&self) -> Range<usize> {
        let start = self.font_direction_hint_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
    fn glyph_data_format_byte_range(&self) -> Range<usize> {
        let start = self.index_to_loc_format_byte_range().end;
        start..start + i16::RAW_BYTE_LEN
    }
}

impl<'a> FontRead<'a> for Head<'a> {
    fn read(data: FontData<'a>) -> Result<Self, ReadError> {
        let mut cursor = data.cursor();
        cursor.advance::<MajorMinor>();
        cursor.advance::<Fixed>();
        cursor.advance::<u32>();
        cursor.advance::<u32>();
        cursor.advance::<u16>();
        cursor.advance::<u16>();
        cursor.advance::<LongDateTime>();
        cursor.advance::<LongDateTime>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<u16>();
        cursor.advance::<u16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.advance::<i16>();
        cursor.finish(HeadMarker {})
    }
}

/// <https://docs.microsoft.com/en-us/typography/opentype/spec/head>
pub type Head<'a> = TableRef<'a, HeadMarker>;

impl<'a> Head<'a> {
    /// Version number of the font header table, set to (1, 0)
    pub fn version(&self) -> MajorMinor {
        let range = self.shape.version_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Set by font manufacturer.
    pub fn font_revision(&self) -> Fixed {
        let range = self.shape.font_revision_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// To compute: set it to 0, sum the entire font as uint32, then
    /// store 0xB1B0AFBA - sum. If the font is used as a component in a
    /// font collection file, the value of this field will be
    /// invalidated by changes to the file structure and font table
    /// directory, and must be ignored.
    pub fn checksum_adjustment(&self) -> u32 {
        let range = self.shape.checksum_adjustment_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Set to 0x5F0F3CF5.
    pub fn magic_number(&self) -> u32 {
        let range = self.shape.magic_number_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// See the flags enum
    pub fn flags(&self) -> u16 {
        let range = self.shape.flags_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Set to a value from 16 to 16384. Any value in this range is
    /// valid. In fonts that have TrueType outlines, a power of 2 is
    /// recommended as this allows performance optimizations in some
    /// rasterizers.
    pub fn units_per_em(&self) -> u16 {
        let range = self.shape.units_per_em_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Number of seconds since 12:00 midnight that started January 1st
    /// 1904 in GMT/UTC time zone.
    pub fn created(&self) -> LongDateTime {
        let range = self.shape.created_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Number of seconds since 12:00 midnight that started January 1st
    /// 1904 in GMT/UTC time zone.
    pub fn modified(&self) -> LongDateTime {
        let range = self.shape.modified_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Minimum x coordinate across all glyph bounding boxes.
    pub fn x_min(&self) -> i16 {
        let range = self.shape.x_min_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Minimum y coordinate across all glyph bounding boxes.
    pub fn y_min(&self) -> i16 {
        let range = self.shape.y_min_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Maximum x coordinate across all glyph bounding boxes.
    pub fn x_max(&self) -> i16 {
        let range = self.shape.x_max_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Maximum y coordinate across all glyph bounding boxes.
    pub fn y_max(&self) -> i16 {
        let range = self.shape.y_max_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// see somewhere else
    pub fn mac_style(&self) -> u16 {
        let range = self.shape.mac_style_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Smallest readable size in pixels.
    pub fn lowest_rec_ppem(&self) -> u16 {
        let range = self.shape.lowest_rec_ppem_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// Deprecated (Set to 2).
    pub fn font_direction_hint(&self) -> i16 {
        let range = self.shape.font_direction_hint_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// 0 for short offsets (Offset16), 1 for long (Offset32).
    pub fn index_to_loc_format(&self) -> i16 {
        let range = self.shape.index_to_loc_format_byte_range();
        self.data.read_at(range.start).unwrap()
    }

    /// 0 for current format.
    pub fn glyph_data_format(&self) -> i16 {
        let range = self.shape.glyph_data_format_byte_range();
        self.data.read_at(range.start).unwrap()
    }
}

#[cfg(feature = "traversal")]
impl<'a> SomeTable<'a> for Head<'a> {
    fn type_name(&self) -> &str {
        "Head"
    }
    fn get_field(&self, idx: usize) -> Option<Field<'a>> {
        match idx {
            0usize => Some(Field::new("version", self.version())),
            1usize => Some(Field::new("font_revision", self.font_revision())),
            2usize => Some(Field::new(
                "checksum_adjustment",
                self.checksum_adjustment(),
            )),
            3usize => Some(Field::new("magic_number", self.magic_number())),
            4usize => Some(Field::new("flags", self.flags())),
            5usize => Some(Field::new("units_per_em", self.units_per_em())),
            6usize => Some(Field::new("created", self.created())),
            7usize => Some(Field::new("modified", self.modified())),
            8usize => Some(Field::new("x_min", self.x_min())),
            9usize => Some(Field::new("y_min", self.y_min())),
            10usize => Some(Field::new("x_max", self.x_max())),
            11usize => Some(Field::new("y_max", self.y_max())),
            12usize => Some(Field::new("mac_style", self.mac_style())),
            13usize => Some(Field::new("lowest_rec_ppem", self.lowest_rec_ppem())),
            14usize => Some(Field::new(
                "font_direction_hint",
                self.font_direction_hint(),
            )),
            15usize => Some(Field::new(
                "index_to_loc_format",
                self.index_to_loc_format(),
            )),
            16usize => Some(Field::new("glyph_data_format", self.glyph_data_format())),
            _ => None,
        }
    }
}

#[cfg(feature = "traversal")]
impl<'a> std::fmt::Debug for Head<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn SomeTable<'a>).fmt(f)
    }
}
