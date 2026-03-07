use std::collections::VecDeque;

/// Maximum number of values retained per pinned widget.
pub const MAX_PIN_VALUES: usize = 1000;

/// Identifies which view occupies the main area.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveView {
    LogList,
    Lens,
}

/// Available layout presets for the Lens grid.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LensLayout {
    Column,
    Grid2,
    Grid3,
    Mosaic,
}

impl LensLayout {
    pub fn css_class(&self) -> &'static str {
        match self {
            LensLayout::Column => "lens-grid column",
            LensLayout::Grid2 => "lens-grid grid2",
            LensLayout::Grid3 => "lens-grid grid3",
            LensLayout::Mosaic => "lens-grid mosaic",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            LensLayout::Column => "Colonne",
            LensLayout::Grid2 => "Grille 2×",
            LensLayout::Grid3 => "Grille 3×",
            LensLayout::Mosaic => "Mosaïque",
        }
    }
}

/// A single captured value for a pinned (source, attribute) pair.
#[derive(Clone, Debug, PartialEq)]
pub struct PinnedValue {
    pub timestamp: String,
    pub raw: String,
    /// Raw byte data (only populated for `data_type == "bytes"`).
    pub raw_bytes: Option<Vec<u8>>,
    /// Originating attribute name (for combined RX+TX pins).
    pub value_attribute: Option<String>,
}

/// Data backing one pinned widget in the Lens.
#[derive(Clone, Debug, PartialEq)]
pub struct LensPinData {
    pub source: String,
    pub attribute: String,
    pub data_type: String,
    pub values: VecDeque<PinnedValue>,
    /// When set, the pin also matches this second attribute (e.g. "TX" paired with "RX").
    pub paired_attribute: Option<String>,
}

impl LensPinData {
    pub fn new(source: String, attribute: String, data_type: String) -> Self {
        Self {
            source,
            attribute,
            data_type,
            values: VecDeque::with_capacity(MAX_PIN_VALUES),
            paired_attribute: None,
        }
    }

    /// Creates a new pin that matches two attributes (e.g. RX + TX).
    pub fn new_paired(
        source: String,
        attribute: String,
        paired: String,
        data_type: String,
    ) -> Self {
        Self {
            source,
            attribute,
            data_type,
            values: VecDeque::with_capacity(MAX_PIN_VALUES),
            paired_attribute: Some(paired),
        }
    }

    /// Push a new value, evicting the oldest if at capacity.
    pub fn push_value(
        &mut self,
        timestamp: String,
        raw: String,
        raw_bytes: Option<Vec<u8>>,
        value_attribute: Option<String>,
    ) {
        if self.values.len() >= MAX_PIN_VALUES {
            self.values.pop_front();
        }
        self.values.push_back(PinnedValue {
            timestamp,
            raw,
            raw_bytes,
            value_attribute,
        });
    }

    /// Returns true if this pin matches the given (source, attribute) pair.
    pub fn matches(&self, source: &str, attribute: &str) -> bool {
        if self.source != source {
            return false;
        }
        if self.attribute == attribute {
            return true;
        }
        if let Some(ref paired) = self.paired_attribute {
            return paired == attribute;
        }
        false
    }
}
