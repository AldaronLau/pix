// raster.rs    Raster images.
//
// Copyright (c) 2017-2020  Douglas P Lau
// Copyright (c) 2019-2020  Jeron Aldaron Lau
//
use crate::channel::{Ch16, Ch8};
use crate::el::Pixel;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::slice::{from_raw_parts_mut, ChunksExact, ChunksExactMut};

/// Builder for [Raster](struct.Raster.html) images.
///
/// After creating a `RasterBuilder`, finish building a `Raster` using one of
/// the *with_* methods:
/// * [with_clear](struct.RasterBuilder.html#method.with_clear)
/// * [with_color](struct.RasterBuilder.html#method.with_color)
/// * [with_raster](struct.RasterBuilder.html#method.with_raster)
/// * [with_pixels](struct.RasterBuilder.html#method.with_pixels)
/// * [with_u8_buffer](struct.RasterBuilder.html#method.with_u8_buffer)
/// * [with_u16_buffer](struct.RasterBuilder.html#method.with_u16_buffer)
///
/// ### Create a `Raster`
/// ```
/// # use pix::*;
/// let r = RasterBuilder::<SRgb8>::new().with_clear(100, 100);
/// ```
pub struct RasterBuilder<P: Pixel> {
    _pixel: PhantomData<P>,
}

/// Image arranged as a rectangular array of pixels.
///
/// ### Create a `Raster` with a solid color rectangle
/// ```
/// # use pix::*;
/// let mut r = RasterBuilder::<SRgb8>::new().with_clear(10, 10);
/// r.compose_color((2, 4, 3, 3), SRgb8::new(0xFF, 0xFF, 0x00));
/// ```
pub struct Raster<P: Pixel> {
    width: i32,
    height: i32,
    pixels: Box<[P]>,
}

/// `Iterator` of *rows* in a [raster], as slices of [pixel]s.
///
/// This struct is created by the [rows] method of [Raster].
///
/// [pixel]: el/trait.Pixel.html
/// [raster]: struct.Raster.html
/// [rows]: struct.Raster.html#method.rows
pub struct Rows<'a, P: Pixel> {
    chunks: ChunksExact<'a, P>,
}

/// `Iterator` of *rows* in a [raster], as mutable slices of [pixel]s.
///
/// This struct is created by the [rows_mut] method of [Raster].
///
/// [pixel]: el/trait.Pixel.html
/// [raster]: struct.Raster.html
/// [rows_mut]: struct.Raster.html#method.rows_mut
pub struct RowsMut<'a, P: Pixel> {
    chunks: ChunksExactMut<'a, P>,
}

/// Location / dimensions of pixels relative to a [Raster](struct.Raster.html).
///
/// ### Create directly
/// ```
/// # use pix::*;
/// let r0 = Region::new(80, 20, 120, 280);
/// let r1 = r0.intersection((50, 40, 360, 240));
/// ```
/// ### Create from Raster
/// ```
/// # use pix::*;
/// let r = RasterBuilder::<SRgb8>::new().with_clear(100, 100);
/// let reg = r.region(); // (0, 0, 100, 100)
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl<P: Pixel> Into<Box<[P]>> for Raster<P> {
    /// Get internal pixel data as boxed slice.
    fn into(self) -> Box<[P]> {
        self.pixels
    }
}

impl<P: Pixel> Into<Vec<P>> for Raster<P> {
    /// Get internal pixel data as `Vec` of pixels.
    fn into(self) -> Vec<P> {
        self.pixels.into()
    }
}

impl<P: Pixel> Default for RasterBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Pixel> RasterBuilder<P> {
    /// Create a new raster builder.
    ///
    /// * `P` [Pixel](el/trait.Pixel.html) format.
    pub fn new() -> Self {
        let _pixel = PhantomData;
        RasterBuilder { _pixel }
    }

    /// Build a `Raster` with all pixels set to the default value.
    ///
    /// ## Examples
    /// ```
    /// # use pix::*;
    /// let r1 = RasterBuilder::<SGray8>::new().with_clear(20, 20);
    /// let r2 = RasterBuilder::<Mask8>::new().with_clear(64, 64);
    /// let r3 = RasterBuilder::<SRgb16>::new().with_clear(10, 10);
    /// let r4 = RasterBuilder::<SGraya32>::new().with_clear(100, 250);
    /// ```
    pub fn with_clear(self, width: u32, height: u32) -> Raster<P> {
        self.with_color(width, height, P::default())
    }

    /// Build a `Raster` with all pixels set to one color.
    ///
    /// ## Example
    /// ```
    /// # use pix::*;
    /// let clr = SRgb8::new(0x40, 0xAA, 0xBB);
    /// let r = RasterBuilder::<SRgb8>::new().with_color(15, 15, clr);
    /// ```
    pub fn with_color(self, width: u32, height: u32, clr: P) -> Raster<P> {
        let width = i32::try_from(width).unwrap();
        let height = i32::try_from(height).unwrap();
        let len = (width * height) as usize;
        let pixels = vec![clr; len].into_boxed_slice();
        Raster {
            width,
            height,
            pixels,
        }
    }

    /// Build a `Raster` by copying another `Raster`.
    ///
    /// * `S` `Pixel` format of source `Raster`.
    ///
    /// ### Convert from Rgb8 to Rgba16
    /// ```
    /// # use pix::*;
    /// let mut r0 = RasterBuilder::<SRgb8>::new().with_clear(50, 50);
    /// // load pixels into raster
    /// let r1 = RasterBuilder::<SRgba16>::new().with_raster(&r0);
    /// ```
    pub fn with_raster<S>(self, src: &Raster<S>) -> Raster<P>
    where
        S: Pixel,
        P::Chan: From<S::Chan>,
    {
        let mut r = RasterBuilder::new().with_clear(src.width(), src.height());
        r.compose_raster((), src, ());
        r
    }

    /// Build a `Raster` with owned pixel data.  You can get ownership of the
    /// pixel data back from the `Raster` as either a `Vec<P>` or a `Box<[P]>`
    /// by calling `into()`.
    ///
    /// * `B` Owned pixed type (`Vec` or boxed slice).
    /// * `width` Width of `Raster`.
    /// * `height` Height of `Raster`.
    /// * `pixels` Pixel data.
    ///
    /// # Panics
    ///
    /// Panics if `pixels` length is not equal to `width` * `height`.
    ///
    /// ## Example
    /// ```
    /// # use pix::*;
    /// let p = vec![SRgb8::new(255, 0, 255); 16]; // vec of magenta pix
    /// let mut r = RasterBuilder::new()           // convert to raster
    ///     .with_pixels(4, 4, p);
    /// let clr = SRgb8::new(0x00, 0xFF, 0x00);    // green
    /// r.compose_color((2, 0, 1, 3), clr);        // make stripe
    /// let p2 = Into::<Vec<SRgb8>>::into(r);      // convert back to vec
    /// ```
    pub fn with_pixels<B>(self, width: u32, height: u32, pixels: B) -> Raster<P>
    where
        B: Into<Box<[P]>>,
    {
        let width = i32::try_from(width).unwrap();
        let height = i32::try_from(height).unwrap();
        let len = (width * height) as usize;
        let pixels = pixels.into();
        assert_eq!(len, pixels.len());
        Raster {
            width,
            height,
            pixels,
        }
    }

    /// Build a `Raster` from a `u8` buffer.
    ///
    /// * `B` Owned pixed type (`Vec` or boxed slice).
    /// * `width` Width of `Raster`.
    /// * `height` Height of `Raster`.
    /// * `buffer` Buffer of pixel data.
    ///
    /// # Panics
    ///
    /// Panics if `buffer` length is not equal to `width` * `height` *
    /// `std::mem::size_of::<P>()`.
    pub fn with_u8_buffer<B>(
        self,
        width: u32,
        height: u32,
        buffer: B,
    ) -> Raster<P>
    where
        B: Into<Box<[u8]>>,
        P: Pixel<Chan = Ch8>,
    {
        let width = i32::try_from(width).unwrap();
        let height = i32::try_from(height).unwrap();
        let len = (width * height) as usize;
        assert!(len > 0);
        let buffer: Box<[u8]> = buffer.into();
        let capacity = buffer.len();
        assert_eq!(
            len * std::mem::size_of::<P>(),
            capacity * std::mem::size_of::<u8>()
        );
        let slice = Box::<[u8]>::into_raw(buffer);
        let pixels: Box<[P]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut P;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Raster {
            width,
            height,
            pixels,
        }
    }

    /// Build a `Raster` from a `u16` buffer.
    ///
    /// * `B` Owned pixed type (`Vec` or boxed slice).
    /// * `width` Width of `Raster`.
    /// * `height` Height of `Raster`.
    /// * `buffer` Buffer of pixel data (in native-endian byte order).
    ///
    /// # Panics
    ///
    /// Panics if `buffer` length is not equal to `width` * `height` *
    /// `std::mem::size_of::<P>()`.
    pub fn with_u16_buffer<B>(
        self,
        width: u32,
        height: u32,
        buffer: B,
    ) -> Raster<P>
    where
        B: Into<Box<[u16]>>,
        P: Pixel<Chan = Ch16>,
    {
        let width = i32::try_from(width).unwrap();
        let height = i32::try_from(height).unwrap();
        let len = (width * height) as usize;
        assert!(len > 0);
        let buffer: Box<[u16]> = buffer.into();
        let capacity = buffer.len();
        assert_eq!(
            len * std::mem::size_of::<P>(),
            capacity * std::mem::size_of::<u16>()
        );
        let slice = Box::<[u16]>::into_raw(buffer);
        let pixels: Box<[P]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut P;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Raster {
            width,
            height,
            pixels,
        }
    }
}

impl<P: Pixel> Raster<P> {
    /// Get width of `Raster`.
    pub fn width(&self) -> u32 {
        self.width as u32
    }

    /// Get height of `Raster`.
    pub fn height(&self) -> u32 {
        self.height as u32
    }

    /// Clear all pixels to default value.
    pub fn clear(&mut self) {
        for p in self.pixels.iter_mut() {
            *p = P::default();
        }
    }

    /// Get one pixel.
    pub fn pixel(&self, x: i32, y: i32) -> P {
        debug_assert!(x >= 0 && x < self.width);
        debug_assert!(y >= 0 && y < self.height);
        let i = (self.width * y + x) as usize;
        self.pixels[i]
    }

    /// Get a mutable pixel.
    pub fn pixel_mut(&mut self, x: i32, y: i32) -> &mut P {
        debug_assert!(x >= 0 && x < self.width);
        debug_assert!(y >= 0 && y < self.height);
        let i = (self.width * y + x) as usize;
        &mut self.pixels[i]
    }

    /// Get a slice of all pixels.
    pub fn pixels(&self) -> &[P] {
        &self.pixels
    }

    /// Get a mutable slice of all pixels.
    pub fn pixels_mut(&mut self) -> &mut [P] {
        &mut self.pixels
    }

    /// Get an `Iterator` of rows within a `Raster`.
    pub fn rows(&self) -> Rows<P> {
        Rows::new(self)
    }

    /// Get an `Iterator` of mutable rows within a `Raster`.
    pub fn rows_mut(&mut self) -> RowsMut<P> {
        RowsMut::new(self)
    }

    /// Get `Region` of entire `Raster`.
    pub fn region(&self) -> Region {
        Region::new(0, 0, self.width(), self.height())
    }

    /// Get intersection with a `Region`
    pub fn intersection<R>(&self, reg: R) -> Region
    where
        R: Into<Region>,
    {
        let reg = reg.into();
        let x0 = reg.x.max(0);
        let x1 = reg.right().min(self.width);
        let w = (x1 - x0).max(0) as u32;
        let y0 = reg.y.max(0);
        let y1 = reg.bottom().min(self.height);
        let h = (y1 - y0).max(0) as u32;
        Region::new(x0, y0, w, h)
    }

    /// Compose from a source `Pixel` color.
    ///
    /// * `reg` Region within `self` (destination).
    /// * `clr` Source `Pixel` color.
    ///
    /// ### Set entire raster to one color
    /// ```
    /// # use pix::*;
    /// let mut r = RasterBuilder::<SRgb32>::new().with_clear(360, 240);
    /// r.compose_color((), SRgb32::new(0.5, 0.2, 0.8));
    /// ```
    /// ### Set rectangle to solid color
    /// ```
    /// # use pix::*;
    /// let mut r = RasterBuilder::<SRgb8>::new().with_clear(100, 100);
    /// r.compose_color((20, 40, 25, 50), SRgb8::new(0xDD, 0x96, 0x70));
    /// ```
    pub fn compose_color<R, S>(&mut self, reg: R, clr: S)
    where
        R: Into<Region>,
        S: Pixel,
        P::Chan: From<S::Chan>,
    {
        let reg = self.intersection(reg.into());
        let width = reg.width();
        let height = reg.height();
        if width > 0 && height > 0 {
            let s = clr.convert();
            let drows = self.rows_mut().skip(reg.y as usize);
            for drow in drows.take(height as usize) {
                let drow = &mut drow[reg.x as usize..];
                for d in drow.iter_mut().take(width as usize) {
                    *d = s;
                }
            }
        }
    }

    /// Compose from a source `Raster`.
    ///
    /// * `to` Region within `self` (destination).
    /// * `src` Source `Raster`.
    /// * `from` Region within source `Raster`.
    ///
    /// `to` / `from` can be `Region` structs, tuples of (*x*, *y*, *width*,
    /// *height*) or the unit type `()`.  Using `()` has the same result as
    /// `Raster::region()`.
    ///
    /// ```bob
    /// *------------+      *-------------+
    /// |            |      |    *------+ |
    /// | *------+   |      |    |      | |
    /// | |      |   |      |    | from | |
    /// | |  to  |   | <--- |    +------+ |
    /// | +------+   |      |             |
    /// |            |      |     src     |
    /// |    self    |      +-------------+
    /// +------------+
    /// ```
    /// The composed `Region` is clamped to the smaller of `to` and `from` in
    /// both `X` and `Y` dimensions.  Also, `to` and `from` are clipped to
    /// their respective `Raster` dimensions.
    ///
    /// ### Copy part of one `Raster` to another, converting pixel format
    /// ```
    /// # use pix::*;
    /// let mut rgb = RasterBuilder::<SRgb8>::new().with_clear(100, 100);
    /// let gray = RasterBuilder::<SGray16>::new()
    ///     .with_color(5, 5, SGray16::new(0x80));
    /// // ... load image data
    /// rgb.compose_raster((40, 40, 5, 5), &gray, ());
    /// ```
    pub fn compose_raster<R0, S, R1>(
        &mut self,
        to: R0,
        src: &Raster<S>,
        from: R1,
    ) where
        R0: Into<Region>,
        R1: Into<Region>,
        S: Pixel,
        P::Chan: From<S::Chan>,
    {
        let (to, from) = (to.into(), from.into());
        let tx = to.x.min(0).abs();
        let ty = to.y.min(0).abs();
        let fx = from.x.min(0).abs();
        let fy = from.y.min(0).abs();
        let to = self.intersection(to);
        let from = src.intersection(from);
        let width = to.width().min(from.width());
        let height = to.height().min(from.height());
        if width > 0 && height > 0 {
            let to = Region::new(to.x + fx, to.y + fy, width, height);
            let from = Region::new(from.x + tx, from.y + ty, width, height);
            let srows = src.rows().skip(from.y as usize);
            let drows = self.rows_mut().skip(to.y as usize);
            for (drow, srow) in drows.take(height as usize).zip(srows) {
                let drow = &mut drow[to.x as usize..];
                let srow = &srow[from.x as usize..];
                for (d, s) in drow.iter_mut().take(width as usize).zip(srow) {
                    *d = s.convert();
                }
            }
        }
    }

    /// Get view of pixels as a `u8` slice.
    ///
    /// Q: Is this UB when P::Chan is Ch32?
    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            let (prefix, v, suffix) = &self.pixels.align_to::<u8>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<'a, P: Pixel> Rows<'a, P> {
    /// Create a new row `Iterator`.
    fn new(raster: &'a Raster<P>) -> Self {
        let width = usize::try_from(raster.width()).unwrap();
        let chunks = raster.pixels.chunks_exact(width);
        Rows { chunks }
    }
}

impl<'a, P: Pixel> Iterator for Rows<'a, P> {
    type Item = &'a [P];

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next()
    }
}

impl<'a, P: Pixel> RowsMut<'a, P> {
    /// Create a new mutable row `Iterator`.
    fn new(raster: &'a mut Raster<P>) -> Self {
        let width = usize::try_from(raster.width()).unwrap();
        let chunks = raster.pixels.chunks_exact_mut(width);
        RowsMut { chunks }
    }
}

impl<'a, P: Pixel> Iterator for RowsMut<'a, P> {
    type Item = &'a mut [P];

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next()
    }
}

impl From<(i32, i32, u32, u32)> for Region {
    fn from(r: (i32, i32, u32, u32)) -> Self {
        Region::new(r.0, r.1, r.2, r.3)
    }
}

impl From<()> for Region {
    fn from(_: ()) -> Self {
        const MAX: u32 = std::i32::MAX as u32;
        Region::new(0, 0, MAX, MAX)
    }
}

impl Region {
    /// Create a new `Region`
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        let width = i32::try_from(width).unwrap_or(0);
        let height = i32::try_from(height).unwrap_or(0);
        Region {
            x,
            y,
            width,
            height,
        }
    }

    /// Get intersection with another `Region`
    pub fn intersection<R>(self, rhs: R) -> Self
    where
        R: Into<Self>,
    {
        let rhs = rhs.into();
        let x0 = self.x.max(rhs.x);
        let x1 = self.right().min(rhs.right());
        let w = (x1 - x0) as u32;
        let y0 = self.y.max(rhs.y);
        let y1 = self.bottom().min(rhs.bottom());
        let h = (y1 - y0) as u32;
        Region::new(x0, y0, w, h)
    }

    /// Get the width
    fn width(self) -> u32 {
        self.width as u32
    }

    /// Get the height
    fn height(self) -> u32 {
        self.height as u32
    }

    /// Get right side
    fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    /// Get bottom side
    fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod test {
    use super::super::*;
    #[test]
    fn region_size() {
        assert_eq!(std::mem::size_of::<Region>(), 16);
    }
    #[test]
    fn intersect() -> Result<(), ()> {
        let r = Region::new(0, 0, 5, 5);
        assert_eq!(r, Region::new(0, 0, 5, 5));
        assert_eq!(r, r.intersection(Region::new(0, 0, 10, 10)));
        assert_eq!(r, r.intersection(Region::new(-5, -5, 10, 10)));
        assert_eq!(
            Region::new(0, 0, 4, 4),
            r.intersection(Region::new(-1, -1, 5, 5))
        );
        assert_eq!(
            Region::new(1, 2, 1, 3),
            r.intersection(Region::new(1, 2, 1, 100))
        );
        assert_eq!(
            Region::new(2, 1, 3, 1),
            r.intersection(Region::new(2, 1, 100, 1))
        );
        Ok(())
    }
    #[test]
    fn with_buffer_rgb8() {
        let b = vec![
            0xAA,0x00,0x00, 0x00,0x11,0x22, 0x33,0x44,0x55,
            0x00,0xBB,0x00, 0x66,0x77,0x88, 0x99,0xAA,0xBB,
            0x00,0x00,0xCC, 0xCC,0xDD,0xEE, 0xFF,0x00,0x11,
        ];
        let r = RasterBuilder::<SRgb8>::new().with_u8_buffer(3, 3, b);
        let v = vec![
            SRgb8::new(0xAA, 0x00, 0x00), SRgb8::new(0x00, 0x11, 0x22),
            SRgb8::new(0x33, 0x44, 0x55),
            SRgb8::new(0x00, 0xBB, 0x00), SRgb8::new(0x66, 0x77, 0x88),
            SRgb8::new(0x99, 0xAA, 0xBB),
            SRgb8::new(0x00, 0x00, 0xCC), SRgb8::new(0xCC, 0xDD, 0xEE),
            SRgb8::new(0xFF, 0x00, 0x11),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn with_buffer_graya16() {
        let b = vec![
            0x1001,0x5005, 0x1000,0x3002, 0x5004,0x7006,
            0x2002,0x6006, 0x9008,0xB00A, 0xD00C,0xF00E,
            0x3003,0x7007, 0xE00F,0xC00D, 0xA00B,0x8009,
        ];
        let r = RasterBuilder::<SGraya16>::new().with_u16_buffer(3, 3, b);
        let v = vec![
            SGraya16::new(0x1001, 0x5005), SGraya16::new(0x1000, 0x3002),
            SGraya16::new(0x5004, 0x7006),
            SGraya16::new(0x2002, 0x6006), SGraya16::new(0x9008, 0xB00A),
            SGraya16::new(0xD00C, 0xF00E),
            SGraya16::new(0x3003, 0x7007), SGraya16::new(0xE00F, 0xC00D),
            SGraya16::new(0xA00B, 0x8009),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn with_pixels_mask32() {
        let p = vec![
            Mask32::new(0.25), Mask32::new(0.5), Mask32::new(0.75),
            Mask32::new(0.5), Mask32::new(0.6), Mask32::new(0.7),
            Mask32::new(0.85), Mask32::new(0.65), Mask32::new(0.45),
        ];
        let r = RasterBuilder::new().with_pixels(3, 3, p.clone());
        assert_eq!(r.pixels(), &p[..]);
    }
    #[test]
    fn pixel_mut_mask8() {
        let mut r = RasterBuilder::<Mask8>::new().with_clear(3, 3);
        *r.pixel_mut(0, 0) = Mask8::new(0xFF);
        *r.pixel_mut(2, 0) = Mask8::new(0x12);
        *r.pixel_mut(1, 1) = Mask8::new(0x34);
        *r.pixel_mut(0, 2) = Mask8::new(0x56);
        *r.pixel_mut(2, 2) = Mask8::new(0x78);
        let v = vec![
            Mask8::new(0xFF), Mask8::new(0x00), Mask8::new(0x12),
            Mask8::new(0x00), Mask8::new(0x34), Mask8::new(0x00),
            Mask8::new(0x56), Mask8::new(0x00), Mask8::new(0x78),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn pixel_mut_mask16() {
        let mut r = RasterBuilder::<Mask16>::new().with_clear(3, 3);
        *r.pixel_mut(2, 0) = Mask16::new(0x9ABC);
        *r.pixel_mut(1, 1) = Mask16::new(0x5678);
        *r.pixel_mut(0, 2) = Mask16::new(0x1234);
        *r.pixel_mut(0, 0) = Mask16::new(1.0);
        *r.pixel_mut(2, 2) = Mask16::new(0x8080);
        let v = vec![
            Mask16::new(0xFFFF), Mask16::new(0x0000), Mask16::new(0x9ABC),
            Mask16::new(0x0000), Mask16::new(0x5678), Mask16::new(0x0000),
            Mask16::new(0x1234), Mask16::new(0x0000), Mask16::new(0x8080),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn raster_with_color() {
        let r = RasterBuilder::<Hwb8>::new().with_color(3, 3,
            Hwb8::new(0x80, 0, 0));
        let v = vec![Hwb8::new(0x80, 0, 0); 9];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn compose_color_gray8() {
        let mut r = RasterBuilder::<SGray8>::new().with_clear(3, 3);
        r.compose_color((0, 0, 1, 1), SGray8::new(0x23));
        r.compose_color((10, 10, 1, 1), SGray8::new(0x45));
        r.compose_color((1, 1, 10, 10), SGray8::new(0xBB));
        let v = vec![
            SGray8::new(0x23), SGray8::new(0), SGray8::new(0),
            SGray8::new(0), SGray8::new(0xBB), SGray8::new(0xBB),
            SGray8::new(0), SGray8::new(0xBB), SGray8::new(0xBB),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn compose_color_srgb8() {
        let mut r = RasterBuilder::<SRgb8>::new().with_clear(3, 3);
        r.compose_color((2, -1, 3, 4), SRgb8::new(0xCC, 0xAA, 0xBB));
        let v = vec![
            SRgb8::new(0, 0, 0), SRgb8::new(0, 0, 0),
            SRgb8::new(0xCC, 0xAA, 0xBB),
            SRgb8::new(0, 0, 0), SRgb8::new(0, 0, 0),
            SRgb8::new(0xCC, 0xAA, 0xBB),
            SRgb8::new(0, 0, 0), SRgb8::new(0, 0, 0),
            SRgb8::new(0xCC, 0xAA, 0xBB),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn compose_raster_gray() {
        let mut g0 = RasterBuilder::<Gray8>::new().with_clear(3, 3);
        let g1 = RasterBuilder::<Gray8>::new().with_color(3, 3,
            Gray8::new(0x40));
        let g2 = RasterBuilder::<Gray8>::new().with_color(3, 3,
            Gray8::new(0x60));
        let g3 = RasterBuilder::<Gray8>::new().with_color(3, 3,
            Gray8::new(0x80));
        g0.compose_raster((-1, 2, 3, 3), &g1, ());
        g0.compose_raster((2, -1, 3, 3), &g2, ());
        g0.compose_raster((-2, -2, 3, 3), &g3, ());
        let v = vec![
            Gray8::new(0x80), Gray8::new(0x00), Gray8::new(0x60),
            Gray8::new(0x00), Gray8::new(0x00), Gray8::new(0x60),
            Gray8::new(0x40), Gray8::new(0x40), Gray8::new(0x00),
        ];
        assert_eq!(g0.pixels(), &v[..]);
    }
    #[test]
    fn compose_raster_rgb() {
        let mut rgb = RasterBuilder::<SRgb8>::new().with_clear(3, 3);
        let gray = RasterBuilder::<SGray16>::new().with_color(3, 3,
            SGray16::new(0x8000));
        rgb.compose_raster((), &gray, (0, 1, 3, 3));
        let mut v = vec![SRgb8::new(0x80, 0x80, 0x80); 6];
        v.extend_from_slice(&vec![SRgb8::new(0, 0, 0); 3]);
        assert_eq!(rgb.pixels(), &v[..]);
    }
    #[test]
    fn with_raster_rgb() {
        let r = RasterBuilder::<SRgb8>::new().with_clear(50, 50);
        let _ = RasterBuilder::<SRgb16>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgb32>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgba8>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgba16>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgba32>::new().with_raster(&r);
        let _ = RasterBuilder::<SGray8>::new().with_raster(&r);
        let _ = RasterBuilder::<SGray16>::new().with_raster(&r);
        let _ = RasterBuilder::<SGray32>::new().with_raster(&r);
        let _ = RasterBuilder::<SGraya8>::new().with_raster(&r);
        let _ = RasterBuilder::<SGraya16>::new().with_raster(&r);
        let _ = RasterBuilder::<SGraya32>::new().with_raster(&r);
        let _ = RasterBuilder::<Mask8>::new().with_raster(&r);
        let _ = RasterBuilder::<Mask16>::new().with_raster(&r);
        let _ = RasterBuilder::<Mask32>::new().with_raster(&r);
    }
    #[test]
    fn with_raster_mask8() {
        let r = RasterBuilder::<Mask8>::new().with_clear(50, 50);
        let _ = RasterBuilder::<SRgb8>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgb16>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgb32>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgba8>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgba16>::new().with_raster(&r);
        let _ = RasterBuilder::<SRgba32>::new().with_raster(&r);
        let _ = RasterBuilder::<SGray8>::new().with_raster(&r);
        let _ = RasterBuilder::<SGray16>::new().with_raster(&r);
        let _ = RasterBuilder::<SGray32>::new().with_raster(&r);
        let _ = RasterBuilder::<SGraya8>::new().with_raster(&r);
        let _ = RasterBuilder::<SGraya16>::new().with_raster(&r);
        let _ = RasterBuilder::<SGraya32>::new().with_raster(&r);
        let _ = RasterBuilder::<Mask8>::new().with_raster(&r);
        let _ = RasterBuilder::<Mask16>::new().with_raster(&r);
        let _ = RasterBuilder::<Mask32>::new().with_raster(&r);
    }
}
