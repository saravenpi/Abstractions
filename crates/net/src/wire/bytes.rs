//! Reading and writing the fixed-size fields of a record body, in little-endian order.

use glam::{DVec3, Vec3};

use super::Frame;
use super::varint::{put_varint, unzigzag, zigzag};
use crate::pack::{Packed, pack_cells, unpack_cells};

/// Adds `position` to `out`: on a surface three cells as zigzag varints then three `i32`
/// millimetres, else three `f64`.
pub(super) fn put_position(out: &mut Vec<u8>, frame: &Frame, position: DVec3) {
    match frame {
        Frame::Surface(_) => {
            let packed = pack_cells(position);
            for cell in packed.cell {
                put_varint(out, zigzag(cell));
            }
            for mm in packed.mm {
                out.extend_from_slice(&mm.to_le_bytes());
            }
        }
        Frame::Around(_) | Frame::System(_) => {
            for metres in position.to_array() {
                out.extend_from_slice(&metres.to_le_bytes());
            }
        }
    }
}

/// Adds `vector` to `out` as three `f32`.
pub(super) fn put_vec3(out: &mut Vec<u8>, vector: Vec3) {
    for part in vector.to_array() {
        out.extend_from_slice(&part.to_le_bytes());
    }
}

/// The bytes of a body that are not read yet.
pub(super) struct Reader<'a>(pub(super) &'a [u8]);

impl Reader<'_> {
    /// Returns the next `N` bytes, or nothing when fewer are left.
    pub(super) fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (head, rest) = self.0.split_first_chunk::<N>()?;
        self.0 = rest;
        Some(*head)
    }

    /// Returns the number [`put_varint`] wrote, or nothing when bytes are missing or it needs
    /// more than ten bytes.
    fn varint(&mut self) -> Option<u64> {
        let mut value = 0u64;
        for shift in (0..70).step_by(7) {
            let [byte] = self.take()?;
            value |= u64::from(byte & 0x7f).checked_shl(shift)?;
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }

    /// Returns the position [`put_position`] wrote for `frame`, or nothing when bytes are
    /// missing or a number is not finite.
    pub(super) fn position(&mut self, frame: &Frame) -> Option<DVec3> {
        let position = match frame {
            Frame::Surface(_) => {
                let cell = [self.varint()?, self.varint()?, self.varint()?];
                let mm = [self.take()?, self.take()?, self.take()?];
                unpack_cells(Packed {
                    cell: cell.map(unzigzag),
                    mm: mm.map(i32::from_le_bytes),
                })
            }
            Frame::Around(_) | Frame::System(_) => {
                let metres = [self.take()?, self.take()?, self.take()?];
                DVec3::from_array(metres.map(f64::from_le_bytes))
            }
        };
        position.is_finite().then_some(position)
    }

    /// Returns the vector [`put_vec3`] wrote, or nothing when bytes are missing or a number is
    /// not finite.
    pub(super) fn vec3(&mut self) -> Option<Vec3> {
        let parts = [self.take()?, self.take()?, self.take()?];
        let vector = Vec3::from_array(parts.map(f32::from_le_bytes));
        vector.is_finite().then_some(vector)
    }

    /// Returns `value` when every byte was read, or nothing when some are left over.
    pub(super) fn end<T>(self, value: T) -> Option<T> {
        self.0.is_empty().then_some(value)
    }
}
