// lib.rs      Pix crate.
//
// Copyright (c) 2019-2020  Douglas P Lau
// Copyright (c) 2019-2020  Jeron Aldaron Lau
//
//! Pixel and [Raster] image crate.
//!
//! A `Raster` is a rectangular array of pixels.  The [Format] is parameterized
//! by [channel], [alpha mode], [gamma mode] and color model.
//!
//! [alpha mode]: alpha/trait.Mode.html
//! [Channel]: trait.Channel.html
//! [Format]: trait.Format.html
//! [gamma mode]: gamma/trait.Mode.html
//! [Raster]: struct.Raster.html
//!
pub mod alpha;
mod channel;
mod format;
pub mod gamma;
mod gray;
mod mask;
mod palette;
mod private;
mod raster;
mod rgb;

pub use crate::channel::{Ch16, Ch32, Ch8, Channel};
pub use crate::format::Format;
pub use crate::gray::{
    Gray, Gray16, Gray32, Gray8, GrayAlpha16, GrayAlpha16p, GrayAlpha32,
    GrayAlpha32p, GrayAlpha8, GrayAlpha8p, SGray16, SGray32, SGray8,
    SGrayAlpha16, SGrayAlpha16p, SGrayAlpha32, SGrayAlpha32p, SGrayAlpha8,
    SGrayAlpha8p,
};
pub use crate::mask::{Mask, Mask16, Mask32, Mask8};
pub use crate::palette::Palette;
pub use crate::raster::{Raster, RasterBuilder, RasterIter, Region};
pub use crate::rgb::{
    Rgb, Rgb16, Rgb32, Rgb8, Rgba16, Rgba16p, Rgba32, Rgba32p, Rgba8, Rgba8p,
    SRgb16, SRgb32, SRgb8, SRgba16, SRgba16p, SRgba32, SRgba32p, SRgba8,
    SRgba8p,
};
