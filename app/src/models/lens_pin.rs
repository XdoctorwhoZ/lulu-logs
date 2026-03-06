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
}

/// Data backing one pinned widget in the Lens.
#[derive(Clone, Debug, PartialEq)]
pub struct LensPinData {
    pub source: String,
    pub attribute: String,
    pub data_type: String,
    pub values: VecDeque<PinnedValue>,
}

impl LensPinData {
    pub fn new(source: String, attribute: String, data_type: String) -> Self {
        Self {
            source,
            attribute,
            data_type,
            values: VecDeque::with_capacity(MAX_PIN_VALUES),
        }
    }

    /// Push a new value, evicting the oldest if at capacity.
    pub fn push_value(&mut self, timestamp: String, raw: String) {
        if self.values.len() >= MAX_PIN_VALUES {
            self.values.pop_front();
        }
        self.values.push_back(PinnedValue { timestamp, raw });
    }

    /// Returns true if this pin matches the given (source, attribute) pair.
    pub fn matches(&self, source: &str, attribute: &str) -> bool {
        self.source == source && self.attribute == attribute
    }
}
