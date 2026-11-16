//! Chunk bookkeeping: which chunks to build, which to finish this frame and which carry colliders.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::detail::Detail;

/// Chunks around the focus, in every direction, that carry colliders.
const SOLID_RADIUS: i32 = 1;

/// Returns whether chunk `coord` must have colliders while the focus is in chunk `center`.
pub fn needs_collider(coord: IVec2, center: IVec2) -> bool {
    let d = (coord - center).abs();
    d.x <= SOLID_RADIUS && d.y <= SOLID_RADIUS
}

/// Picks the finished chunks to turn into entities now: the `budget` nearest, plus any that need colliders.
pub fn select_batch(mut done: Vec<IVec2>, center: IVec2, budget: usize) -> Vec<IVec2> {
    done.sort_by_key(|c| (*c - center).length_squared());
    done.into_iter()
        .enumerate()
        .filter(|(i, c)| *i < budget || needs_collider(*c, center))
        .map(|(_, c)| c)
        .collect()
}

/// Returns the chunks within `radius` of `center`, nearest first.
pub fn ring(center: IVec2, radius: i32) -> Vec<IVec2> {
    let mut all: Vec<IVec2> = (-radius..=radius)
        .flat_map(|dx| (-radius..=radius).map(move |dz| center + IVec2::new(dx, dz)))
        .filter(|c| (*c - center).length_squared() <= radius * radius)
        .collect();
    all.sort_by_key(|c| (*c - center).length_squared());
    all
}

/// A spawned chunk with the colliders it carries while the player is near.
pub struct Loaded {
    /// The chunk entity: terrain mesh and heightfield.
    pub entity: Entity,
    /// The merged props child, when the chunk has props.
    pub props: Option<Entity>,
    /// The heightfield collider, kept while detached.
    pub heights: Collider,
    /// The compound collider of trunks, cacti and rocks, kept while detached.
    pub bodies: Option<Collider>,
    /// Whether the colliders are currently attached.
    pub solid: bool,
    /// Level of detail, ground cover and structure state.
    pub detail: Detail,
}

impl Loaded {
    /// Attaches or detaches the colliders; does nothing when they are already in that state.
    pub fn set_solid(&mut self, commands: &mut Commands, on: bool) {
        if self.solid == on {
            return;
        }
        self.solid = on;
        let bodies = self.props.zip(self.bodies.as_ref());
        let structure = self.detail.structure();
        match on {
            true => {
                commands
                    .entity(self.entity)
                    .try_insert(self.heights.clone());
                if let Some((e, c)) = bodies {
                    commands.entity(e).try_insert(c.clone());
                }
                if let Some((e, c)) = structure {
                    commands.entity(*e).try_insert(c.clone());
                }
            }
            false => {
                commands.entity(self.entity).try_remove::<Collider>();
                if let Some((e, _)) = bodies {
                    commands.entity(e).try_remove::<Collider>();
                }
                if let Some((e, _)) = structure {
                    commands.entity(*e).try_remove::<Collider>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the 3x3 around the focus needs colliders.
    #[test]
    fn collider_zone_is_three_by_three() {
        let c = IVec2::new(4, -2);
        assert!(needs_collider(c, c) && needs_collider(c + IVec2::new(1, -1), c));
        assert!(!needs_collider(c + IVec2::new(2, 0), c));
    }

    /// The budget takes the nearest chunks, and the 3x3 always goes through.
    #[test]
    fn batch_is_nearest_first_and_keeps_the_solid_zone() {
        let c = IVec2::ZERO;
        let done = vec![
            IVec2::new(5, 0),
            IVec2::new(3, 0),
            IVec2::new(1, 1),
            IVec2::new(0, 4),
        ];
        assert_eq!(select_batch(done.clone(), c, 1), vec![IVec2::new(1, 1)]);
        assert_eq!(
            select_batch(done, c, 2),
            vec![IVec2::new(1, 1), IVec2::new(3, 0)]
        );
    }

    /// The ring is a disc sorted from the center.
    #[test]
    fn ring_is_a_sorted_disc() {
        let r = ring(IVec2::new(2, 2), 2);
        assert_eq!(r[0], IVec2::new(2, 2));
        assert_eq!(r.len(), 13);
        assert!(
            r.windows(2)
                .all(|w| (w[0] - IVec2::new(2, 2)).length_squared()
                    <= (w[1] - IVec2::new(2, 2)).length_squared())
        );
    }
}
