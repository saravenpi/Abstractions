//! What the search for a home has found so far, and the home it settles on.

use bevy::math::DVec3;

use super::home::{Fit, NOWHERE, homely, living, suited};
use super::ids::BodyId;
use super::nudge::{self, Fitted};
use super::spec::StarSpec;
use super::{LY, SECTOR_LY, system};

/// Homely planets the search keeps as the ones a home may be fitted from, the nearest first.
const CANDIDATES: usize = 8;

/// What the search has found so far.
pub(super) struct Found {
    /// The point the search goes out from, in metres from the galaxy centre.
    from: DVec3,
    /// The test a home passes.
    fit: Fit,
    /// Systems looked at before the search settles for less than a home.
    reach: usize,
    /// The nearest planet that passes [`Found::fit`] and the metres from the search point to its
    /// star.
    home: Option<(f64, BodyId)>,
    /// The first [`CANDIDATES`] [`homely`] planets found, with the metres to their star and the
    /// star.
    homely: Vec<(f64, StarSpec, BodyId)>,
    /// The nearest living planet, homely or not, and the metres to its star.
    living: Option<(f64, BodyId)>,
    /// The planet most likely to hold life and that chance.
    spare: Option<(f32, BodyId)>,
    /// How many systems were looked at.
    seen: usize,
}

impl Found {
    /// Returns a search from `from` metres off the galaxy centre for a home that passes `fit`,
    /// over `reach` systems at most when none does.
    pub(super) fn new(from: DVec3, fit: Fit, reach: usize) -> Self {
        Self {
            from,
            fit,
            reach,
            home: None,
            homely: Vec::with_capacity(CANDIDATES),
            living: None,
            spare: None,
            seen: 0,
        }
    }

    /// Looks at the system of `star`, unless a home nearer than it is already known.
    pub(super) fn look(&mut self, star: &StarSpec) {
        let out = star.pos.distance(self.from);
        if self.home.is_some_and(|(nearest, _)| out >= nearest) {
            return;
        }
        let planets = system::planets(star);
        self.seen += 1;
        let at = |id: BodyId| (out, id);
        self.home = living(star, &planets, Some(self.fit)).map(at).or(self.home);
        if self.homely.len() < CANDIDATES
            && let Some(id) = living(star, &planets, Some(homely))
        {
            self.homely.push((out, star.clone(), id));
        }
        if self.living.is_none() {
            self.living = living(star, &planets, None).map(at);
        }
        let spare = suited(star, &planets);
        if spare.is_some_and(|(chance, _)| self.spare.is_none_or(|(best, _)| chance > best)) {
            self.spare = spare;
        }
    }

    /// Returns true when no shell from `low` light-years out can change the answer: the
    /// nearest home is closer than any star left, or [`Found::reach`] systems had none.
    pub(super) fn settled(&self, low: f64) -> bool {
        let slack = 0.5 * 3f64.sqrt() * SECTOR_LY;
        let nearer = |(out, _): (f64, BodyId)| out < (low - slack) * LY;
        self.home.map_or(self.seen >= self.reach, nearer)
    }

    /// Returns the home the search settles on and how it was fitted: the home it found; failing
    /// that, when `fitting`, the nearest homely planet that can be fitted to the Earth, see
    /// [`nudge::nearest`]; failing that the nearest homely planet, the first living one, and the
    /// landable planet most likely to hold life.
    pub(super) fn answer(mut self, fitting: bool) -> (BodyId, Option<Fitted>) {
        if let Some((_, id)) = self.home {
            return (id, None);
        }
        self.homely.sort_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((id, fit)) = fitting.then(|| nudge::nearest(&self.homely)).flatten() {
            return (id, Some(fit));
        }
        let homely = self.homely.first().map(|(out, _, id)| (*out, *id));
        let chosen = homely.or(self.living).map(|(_, id)| id);
        (
            chosen.or(self.spare.map(|(_, id)| id)).unwrap_or(NOWHERE),
            None,
        )
    }
}
