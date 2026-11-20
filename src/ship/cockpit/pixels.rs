//! A small black and white picture: the canvas a display is painted on, and its bytes.

/// Colour of a lit pixel and of a dark one, as sRGB bytes. The material of the display
/// multiplies both by its brightness.
const INK: ([u8; 4], [u8; 4]) = ([255, 255, 255, 255], [6, 9, 14, 255]);

/// A canvas of pixels, each lit or dark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    /// Pixels per row.
    pub width: usize,
    /// Rows.
    pub height: usize,
    /// The pixels, row by row from the top left.
    lit: Vec<bool>,
}

impl Pixels {
    /// Returns a dark canvas of `width` by `height` pixels.
    pub fn dark(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            lit: vec![false; width * height],
        }
    }

    /// Lights the pixel at column `x` and row `y`. A pixel outside the canvas is dropped.
    pub fn light(&mut self, x: i32, y: i32) {
        if (0..self.width as i32).contains(&x) && (0..self.height as i32).contains(&y) {
            self.lit[y as usize * self.width + x as usize] = true;
        }
    }

    #[cfg(test)]
    /// Returns whether the pixel at column `x` and row `y` is lit.
    pub fn is_lit(&self, x: i32, y: i32) -> bool {
        (0..self.width as i32).contains(&x)
            && (0..self.height as i32).contains(&y)
            && self.lit[y as usize * self.width + x as usize]
    }

    #[cfg(test)]
    /// Returns how many pixels are lit.
    pub fn count(&self) -> usize {
        self.lit.iter().filter(|lit| **lit).count()
    }

    /// Returns the picture as sRGB RGBA bytes, row by row from the top left.
    pub fn rgba(&self) -> Vec<u8> {
        let pick = |lit: &bool| if *lit { INK.0 } else { INK.1 };
        self.lit.iter().flat_map(pick).collect()
    }
}
