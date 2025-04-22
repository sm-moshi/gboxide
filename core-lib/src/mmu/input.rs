//! Game Boy input button definitions and helpers
//
//! This module provides a type-safe enum for Game Boy buttons and helpers for mapping.

/// Game Boy buttons, in hardware order (Right, Left, Up, Down, A, B, Select, Start)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameBoyButton {
    /// D-Pad Right
    Right = 0,
    /// D-Pad Left
    Left = 1,
    /// D-Pad Up
    Up = 2,
    /// D-Pad Down
    Down = 3,
    /// Button A
    A = 4,
    /// Button B
    B = 5,
    /// Button Select
    Select = 6,
    /// Button Start
    Start = 7,
}

impl GameBoyButton {
    /// Convert from a button index (0-7) to a `GameBoyButton`, if valid.
    pub const fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Right),
            1 => Some(Self::Left),
            2 => Some(Self::Up),
            3 => Some(Self::Down),
            4 => Some(Self::A),
            5 => Some(Self::B),
            6 => Some(Self::Select),
            7 => Some(Self::Start),
            _ => None,
        }
    }
    /// Convert from a `GameBoyButton` to its hardware index (0-7)
    pub const fn to_index(self) -> usize {
        self as usize
    }
}
