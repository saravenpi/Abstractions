//! A small picture in a few inks: the canvas a display is painted on, and its bytes.

/// What a pixel of a display is painted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// Nothing: the glass of the display.
    Dark,
    /// The grey of a legend whose lamp is out.
    Dim,
    /// The white of a reading.
    Lit,
    /// The amber of a warning.
    Amber,
    /// The red of a danger.
    Red,
}

/// Colour of each ink in the order of [`Ink`], as sRGB bytes. The material of the display
/// multiplies them by its brightness.
const INKS: [[u8; 4]; 5] = [
    [6, 9, 14, 255],
    [84, 96, 116, 255],
    [255, 255, 255, 255],
    [255, 160, 30, 255],
    [255, 50, 40, 255],
];

impl Ink {
    /// Returns the sRGB bytes of the ink.
    pub fn rgba(self) -> [u8; 4] {
        INKS[self as usize]
    }
}

/// A canvas of pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    /// Pixels per row.
    pub width: usize,
    /// Rows.
    pub height: usize,
    /// The pixels, row by row from the top left.
    inks: Vec<Ink>,
}

impl Pixels {
    /// Returns a dark canvas of `width` by `height` pixels.
    pub fn dark(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            inks: vec![Ink::Dark; width * height],
        }
    }

    /// Paints the pixel at column `x` and row `y` with `ink`. A pixel outside the canvas is
    /// dropped.
    pub fn put(&mut self, x: i32, y: i32, ink: Ink) {
        if (0..self.width as i32).contains(&x) && (0..self.height as i32).contains(&y) {
            self.inks[y as usize * self.width + x as usize] = ink;
        }
    }

    /// Returns the ink of the pixel at column `x` and row `y`, dark outside the canvas.
    #[cfg(test)]
    pub fn ink(&self, x: i32, y: i32) -> Ink {
        let inside = (0..self.width as i32).contains(&x) && (0..self.height as i32).contains(&y);
        match inside {
            true => self.inks[y as usize * self.width + x as usize],
            false => Ink::Dark,
        }
    }

    /// Returns whether the pixel at column `x` and row `y` is painted.
    #[cfg(test)]
    pub fn is_lit(&self, x: i32, y: i32) -> bool {
        self.ink(x, y) != Ink::Dark
    }

    /// Returns how many pixels are painted.
    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.inks.iter().filter(|ink| **ink != Ink::Dark).count()
    }

    /// Returns the picture as sRGB RGBA bytes, row by row from the top left.
    pub fn rgba(&self) -> Vec<u8> {
        self.inks.iter().flat_map(|ink| ink.rgba()).collect()
    }
}
