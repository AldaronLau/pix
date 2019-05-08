// channel.rs       Color channels
//
// Copyright (c) 2019  Douglas P Lau
//
use std::cmp::Ordering;
use std::ops::{Div, Mul};
use crate::gamma::Gamma;

/// One *component* of a pixel [Format](trait.Format.html).
///
/// For example, in [Rgb](struct.Rgb.html) there are *red*, *green* and *blue*
/// channels.
///
/// Defined channels are [Ch8](struct.Ch8.html), [Ch16](struct.Ch16.html)
/// and [Ch32](struct.Ch32.html).
pub trait Channel: Copy + Default + Ord + Mul<Output=Self> + Div<Output=Self> +
    From<u8> + Into<u8> + Gamma
{
    /// Minimum intensity (*zero*)
    const MIN: Self;

    /// Maximum intensity (*one*)
    const MAX: Self;

    /// Linear interpolation with alpha
    fn lerp_alpha(self, dest: Self, alpha: Self) -> Self;
}

/// 8-bit color [Channel](trait.Channel.html).
///
/// The channel is represented by a u8, but multiplication and division treat
/// the values as though they range between 0 and 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ch8(u8);

impl Ch8 {
    /// Create a new 8-bit channel value.
    pub fn new(value: u8) -> Self {
        Ch8 { 0: value }
    }
}

impl From<u8> for Ch8 {
    fn from(value: u8) -> Self {
        Ch8 { 0: value }
    }
}

impl From<f32> for Ch8 {
    /// Convert from an f32.
    ///
    /// Returns [MIN](trait.Channel.html#associatedconstant.MIN) if value is
    ///         less than 0.0, or NaN.
    /// Returns [MAX](trait.Channel.html#associatedconstant.MAX) if value is
    ///         greater than 1.0.
    fn from(value: f32) -> Self {
        // checks here to avoid UB on float-to-int cast (bug #10184)
        let v = if value.is_nan() || value < 0.0 {
            0
        } else if value > 1.0 {
            255
        } else {
            (value * 255.0).round() as u8
        };
        Ch8 { 0: v }
    }
}

impl From<Ch8> for u8 {
    fn from(c: Ch8) -> u8 {
        c.0
    }
}

impl Mul for Ch8 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let l = self.0 as u32;
        let l = (l << 4) | (l >> 4);
        let r = rhs.0 as u32;
        let r = (r << 4) | (r >> 4);
        let value = ((l * r) >> 16) as u8;
        Ch8 { 0: value }
    }
}

impl Div for Ch8 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.0 > 0 {
            let ss = (self.0 as u32) << 8;
            let rr = rhs.0 as u32;
            let value = (ss / rr).min(255) as u8;
            Ch8 { 0: value }
        } else {
            Ch8 { 0: 0 }
        }
    }
}

impl Gamma for Ch8 {
    /// Encode a gamma value from linear intensity
    fn encode_gamma(self) -> Self {
        Ch8 { 0: self.0.encode_gamma() }
    }
    /// Decode a gamma value into linear intensity
    fn decode_gamma(self) -> Self {
        Ch8 { 0: self.0.decode_gamma() }
    }
}

impl Channel for Ch8 {

    /// Minimum intensity (*zero*)
    const MIN: Ch8 = Ch8 { 0: 0 };

    /// Maximum intensity (*one*)
    const MAX: Ch8 = Ch8 { 0: 0xFF };

    /// Linear interpolation
    #[inline]
    fn lerp_alpha(self, dest: Self, alpha: Self) -> Self {
        // NOTE: Alpha blending euqation is: `alpha * top + (1 - alpha) * bot`
        //       This is equivalent to lerp: `bot + alpha * (top - bot)`
        let top: i32 = self.0.into();
        let bot: i32 = dest.0.into();
        let r = bot + scale_i32(alpha.0, top - bot);
        Ch8 { 0: r as u8 }
    }
}

/// Scale an i32 value by a u8 (for alpha blending)
#[inline]
fn scale_i32(a: u8, v: i32) -> i32 {
    let c = v * a as i32;
    // cheap alternative to divide by 255
    (((c + 1) + (c >> 8)) >> 8) as i32
}

/// 16-bit color [Channel](trait.Channel.html)
///
/// The channel is represented by a u16, but multiplication and division treat
/// the values as though they range between 0 and 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ch16(u16);

impl Ch16 {
    /// Create a new 16-bit channel value.
    pub fn new(value: u16) -> Self {
        Ch16 { 0: value }
    }
}

impl From<u8> for Ch16 {
    fn from(value: u8) -> Self {
        let value = value as u16;
        let value = value << 8 | value;
        Ch16 { 0: value }
    }
}

impl From<f32> for Ch16 {
    fn from(value: f32) -> Self {
        // assert needed here to avoid UB on float-to-int cast
        // once bug #10184 is fixed, this can be removed
        assert!(value >= 0.0 && value <= 1.0);
        let value = (value * 65535.0).round() as u16;
        Ch16 { 0: value }
    }
}

impl From<Ch16> for u8 {
    fn from(c: Ch16) -> u8 {
        (c.0 >> 8) as u8
    }
}

impl Mul for Ch16 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let l = self.0 as u64;
        let l = (l << 8) | (l >> 8);
        let r = rhs.0 as u64;
        let r = (r << 8) | (r >> 8);
        let value = ((l * r) >> 32) as u16;
        Ch16 { 0: value }
    }
}

impl Div for Ch16 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.0 > 0 {
            let ss = (self.0 as u64) << 16;
            let rr = rhs.0 as u64;
            let value = (ss / rr).min(65535) as u16;
            Ch16 { 0: value }
        } else {
            Ch16 { 0: 0 }
        }
    }
}

impl Gamma for Ch16 {
    /// Encode a gamma value from linear intensity
    fn encode_gamma(self) -> Self {
        Ch16 { 0: self.0.encode_gamma() }
    }
    /// Decode a gamma value into linear intensity
    fn decode_gamma(self) -> Self {
        Ch16 { 0: self.0.decode_gamma() }
    }
}

impl Channel for Ch16 {

    /// Minimum intensity (*zero*)
    const MIN: Ch16 = Ch16 { 0: 0 };

    /// Maximum intensity (*one*)
    const MAX: Ch16 = Ch16 { 0: 0xFFFF };

    /// Linear interpolation
    #[inline]
    fn lerp_alpha(self, dest: Self, alpha: Self) -> Self {
        // NOTE: Alpha blending euqation is: `alpha * top + (1 - alpha) * bot`
        //       This is equivalent to lerp: `bot + alpha * (top - bot)`
        let top: i32 = self.0.into();
        let bot: i32 = dest.0.into();
        let r = bot + scale_i32(alpha.into(), top - bot);
        Ch16 { 0: r as u16 }
    }
}

/// 32-bit color [Channel](trait.Channel.html)
///
/// The channel is represented by an f32, but the value is guaranteed to be
/// between 0 and 1, inclusive.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Ch32(f32);

impl Ch32 {
    /// Create a new 32-bit channel value.
    ///
    /// Returns [MIN](trait.Channel.html#associatedconstant.MIN) if value is
    ///         less than 0.0, or NaN.
    /// Returns [MAX](trait.Channel.html#associatedconstant.MAX) if value is
    ///         greater than 1.0.
    pub fn new(value: f32) -> Self {
        let v = if value.is_nan() || value < 0.0 {
            0.0
        } else if value > 1.0 {
            1.0
        } else {
            value
        };
        Ch32 { 0: v }
    }
}

impl From<u8> for Ch32 {
    fn from(value: u8) -> Self {
        let value = value as f32 * 255.0;
        Ch32 { 0: value }
    }
}

impl From<f32> for Ch32 {
    fn from(value: f32) -> Self {
        Ch32::new(value)
    }
}

impl From<Ch32> for u8 {
    fn from(c: Ch32) -> u8 {
        let value = c.0;
        debug_assert!(value >= 0.0 && value <= 1.0);
        // cast is not UB since the value is guaranteed to
        // be between 0.0 and 1.0 (see bug #10184)
        (value * 255.0).round() as u8
    }
}

impl Eq for Ch32 { }

impl Ord for Ch32 {
    fn cmp(&self, other: &Ch32) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Mul for Ch32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Ch32 { 0: self.0 * rhs.0 }
    }
}

impl Div for Ch32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let v = rhs.0;
        if v > 0.0 {
            let value = (self.0 / v).min(1.0);
            Ch32 { 0: value }
        } else {
            Ch32 { 0: 0.0 }
        }
    }
}

impl Gamma for Ch32 {
    /// Encode a gamma value from linear intensity
    fn encode_gamma(self) -> Self {
        Ch32 { 0: self.0.encode_gamma() }
    }
    /// Decode a gamma value into linear intensity
    fn decode_gamma(self) -> Self {
        Ch32 { 0: self.0.decode_gamma() }
    }
}

impl Channel for Ch32 {

    /// Minimum intensity (*zero*)
    const MIN: Ch32 = Ch32 { 0: 0.0 };

    /// Maximum intensity (*one*)
    const MAX: Ch32 = Ch32 { 0: 1.0 };

    /// Linear interpolation
    #[inline]
    fn lerp_alpha(self, dest: Self, alpha: Self) -> Self {
        // NOTE: Alpha blending euqation is: `alpha * top + (1 - alpha) * bot`
        //       This is equivalent to lerp: `bot + alpha * (top - bot)`
        let top = self.0;
        let bot = dest.0;
        let v = bot + alpha.0 * (top - bot);
        Ch32 { 0: v }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ch8_into() {
        assert_eq!(Ch8::new(255), 1.0.into());
        assert_eq!(Ch8::new(128), 0.5.into());
        assert_eq!(Ch8::new(64), 0.25.into());
        assert_eq!(Ch8::new(32), 0.125.into());
    }
    #[test]
    fn ch8_mul() {
        assert_eq!(Ch8::new(255), Ch8::new(255) * 1.0.into());
        assert_eq!(Ch8::new(128), Ch8::new(255) * 0.5.into());
        assert_eq!(Ch8::new(64), Ch8::new(255) * 0.25.into());
        assert_eq!(Ch8::new(32), Ch8::new(255) * 0.125.into());
        assert_eq!(Ch8::new(16), Ch8::new(255) * 0.0625.into());
        assert_eq!(Ch8::new(64), Ch8::new(128) * 0.5.into());
        assert_eq!(Ch8::new(32), Ch8::new(128) * 0.25.into());
        assert_eq!(Ch8::new(16), Ch8::new(128) * 0.125.into());
        assert_eq!(Ch8::new(8), Ch8::new(128) * 0.0625.into());
    }
    #[test]
    fn ch8_div() {
        assert_eq!(Ch8::new(255), Ch8::new(255) / 1.0.into());
        assert_eq!(Ch8::new(255), Ch8::new(128) / 0.5.into());
        assert_eq!(Ch8::new(255), Ch8::new(64) / 0.25.into());
        assert_eq!(Ch8::new(255), Ch8::new(32) / 0.125.into());
        assert_eq!(Ch8::new(255), Ch8::new(16) / 0.0625.into());
        assert_eq!(Ch8::new(128), Ch8::new(128) / 1.0.into());
        assert_eq!(Ch8::new(128), Ch8::new(64) / 0.5.into());
        assert_eq!(Ch8::new(128), Ch8::new(32) / 0.25.into());
        assert_eq!(Ch8::new(128), Ch8::new(16) / 0.125.into());
        assert_eq!(Ch8::new(64), Ch8::new(64) / 1.0.into());
        assert_eq!(Ch8::new(64), Ch8::new(32) / 0.5.into());
        assert_eq!(Ch8::new(64), Ch8::new(16) / 0.25.into());
    }
    #[test]
    fn ch16_into() {
        assert_eq!(Ch16::new(65535), 1.0.into());
        assert_eq!(Ch16::new(32768), 0.5.into());
        assert_eq!(Ch16::new(16384), 0.25.into());
        assert_eq!(Ch16::new(8192), 0.125.into());
    }
    #[test]
    fn ch16_mul() {
        assert_eq!(Ch16::new(65535), Ch16::new(65535) * 1.0.into());
        assert_eq!(Ch16::new(32768), Ch16::new(65535) * 0.5.into());
        assert_eq!(Ch16::new(16384), Ch16::new(65535) * 0.25.into());
        assert_eq!(Ch16::new(8192), Ch16::new(65535) * 0.125.into());
        assert_eq!(Ch16::new(4096), Ch16::new(65535) * 0.0625.into());
        assert_eq!(Ch16::new(16384), Ch16::new(32768) * 0.5.into());
        assert_eq!(Ch16::new(8192), Ch16::new(32768) * 0.25.into());
        assert_eq!(Ch16::new(4096), Ch16::new(32768) * 0.125.into());
        assert_eq!(Ch16::new(2048), Ch16::new(32768) * 0.0625.into());
    }
    #[test]
    fn ch16_div() {
        assert_eq!(Ch16::new(65535), Ch16::new(65535) / 1.0.into());
        assert_eq!(Ch16::new(65535), Ch16::new(32768) / 0.5.into());
        assert_eq!(Ch16::new(65535), Ch16::new(16384) / 0.25.into());
        assert_eq!(Ch16::new(65535), Ch16::new(8192) / 0.125.into());
        assert_eq!(Ch16::new(65535), Ch16::new(4096) / 0.0625.into());
        assert_eq!(Ch16::new(32768), Ch16::new(32768) / 1.0.into());
        assert_eq!(Ch16::new(32768), Ch16::new(16384) / 0.5.into());
        assert_eq!(Ch16::new(32768), Ch16::new(8192) / 0.25.into());
        assert_eq!(Ch16::new(32768), Ch16::new(4096) / 0.125.into());
        assert_eq!(Ch16::new(16384), Ch16::new(16384) / 1.0.into());
        assert_eq!(Ch16::new(16384), Ch16::new(8192) / 0.5.into());
        assert_eq!(Ch16::new(16384), Ch16::new(4096) / 0.25.into());
    }
}
