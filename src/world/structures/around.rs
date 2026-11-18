//! What stands around a point of a planet: the sites and built floors within reach, surveyed once off the
//! main thread and then read every frame for nothing.

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::geo::stretch;
use super::pads::{FloorKind, Nearby, Pad, Strip, site_pads};
use super::site::{CELL, Kind, Site, structure_in_cell};
use super::village;
use crate::world::Terrain;
use crate::world::focus::Focus;
use crate::world::globe::{Chart, Face};

mod rooms;

/// Radius in meters around the focus within which [`Surroundings`] always knows every site.
pub const AROUND: f32 = 320.0;

/// How far the focus may move from the middle of a survey before a new one starts, in meters.
const DRIFT: f32 = 64.0;

/// How much wider than asked, in chart units per meter, the cells of a survey are searched.
const SLACK: f32 = 1.3;

/// The sites and built floors around one point of a planet. Plain data in the frame of the planet.
#[derive(Default, Clone)]
pub struct Survey {
    planet: (u64, u64, bool),
    center: DVec3,
    sites: Vec<Nearby>,
    pads: Vec<Pad>,
    strips: Vec<Strip>,
}

impl Survey {
    /// Returns the nearest surveyed structure or village whose origin lies within `radius` meters of the
    /// planet position `pos`.
    pub fn nearest(&self, pos: DVec3, radius: f32) -> Option<Nearby> {
        let seen = self.sites.iter().map(|s| Nearby {
            distance: s.pos.distance(pos) as f32,
            ..*s
        });
        let near = seen.filter(|s| s.distance <= radius);
        near.min_by(|a, b| a.distance.total_cmp(&b.distance))
    }

    /// Returns what the built floor under feet at the planet position `pos` is made of, or nothing when
    /// they stand on bare ground. House floors and platforms come before streets and paths.
    pub fn floor_at(&self, pos: DVec3) -> Option<FloorKind> {
        let pads = self.pads.iter().find_map(|p| p.under(pos));
        pads.or_else(|| self.strips.iter().find_map(|s| s.under(pos)))
    }

    /// Adds the site `site` of the chart `face` when it reaches within `radius` meters of the center.
    fn add(&mut self, face: &Terrain, site: &Site, radius: f32) {
        let view = face.around(site.origin.x, site.origin.y);
        let pos = view.embed(site.origin.x, site.floor, site.origin.y);
        let (reach, distance) = (site.kind.reach(), pos.distance(self.center) as f32);
        if distance > radius + reach {
            return;
        }
        self.sites.push(Nearby {
            kind: site.kind,
            pos,
            reach,
            distance,
        });
        self.pads.extend(site_pads(&view, site));
        if site.kind == Kind::Village {
            let (pads, strips) = village::floors(&view, site);
            self.pads.extend(pads);
            self.strips.extend(strips);
        }
    }

    /// Searches the cells of the chart `face` within `radius` meters of the center.
    fn scan(&mut self, face: &Terrain, radius: f32) {
        let at = face.locate(self.center).xz();
        let pad = (radius + village::REACH) * stretch(face, at) * SLACK;
        let bound = match face.chart() {
            Chart::Face { scale, .. } => scale as f32,
            _ => f32::MAX,
        };
        let lo = ((at - pad).max(Vec2::splat(-bound)) / CELL)
            .floor()
            .as_ivec2();
        let hi = ((at + pad).min(Vec2::splat(bound)) / CELL)
            .floor()
            .as_ivec2();
        for cell in (lo.x..=hi.x).flat_map(|x| (lo.y..=hi.y).map(move |y| IVec2::new(x, y))) {
            if let Some(site) = structure_in_cell(face, cell) {
                self.add(face, &site, radius);
            }
        }
    }
}

/// Surveys the sites and built floors within `radius` meters of the planet position `center` of the planet
/// of `terrain`, on every cube face in reach. It searches cells and lays out villages, which takes
/// milliseconds: call it off the main thread, or read [`Surroundings`] instead. Runs on any thread and reads
/// no world.
pub fn survey(terrain: &Terrain, center: DVec3, radius: f32) -> Survey {
    let globe = terrain.globe();
    let mut out = Survey {
        planet: (
            globe.seed(),
            globe.radius().to_bits(),
            globe.reserve().is_some(),
        ),
        center,
        ..default()
    };
    if matches!(terrain.chart(), Chart::Flat) {
        out.scan(terrain, radius);
        return out;
    }
    let dir = center.normalize_or_zero();
    for face in Face::ALL.into_iter().filter(|f| f.axes()[0].dot(dir) > 0.5) {
        out.scan(&Terrain::on(globe.clone(), globe.face_chart(face)), radius);
    }
    out
}

/// The sites and built floors within [`AROUND`] meters of the player, kept up to date by a background task.
/// Reading it costs a walk over a short list and allocates nothing.
#[derive(Resource, Default)]
pub struct Surroundings {
    survey: Survey,
    task: Option<Task<Survey>>,
}

impl Surroundings {
    /// Returns the nearest structure or village whose origin lies within `radius` meters of the planet
    /// position `pos`. Complete while `pos` is within [`AROUND`] minus `radius` meters of the player.
    pub fn nearest(&self, pos: DVec3, radius: f32) -> Option<Nearby> {
        self.survey.nearest(pos, radius)
    }

    /// Returns what the built floor under feet at the planet position `pos` is made of, or nothing on bare
    /// ground. Complete within [`AROUND`] meters of the player.
    pub fn floor_at(&self, pos: DVec3) -> Option<FloorKind> {
        self.survey.floor_at(pos)
    }
}

/// Keeps [`Surroundings`] around the streaming focus: takes a finished survey, and starts a new one on a
/// worker thread when the focus has drifted or the planet has changed.
pub fn watch(around: ResMut<Surroundings>, terrain: Option<Res<Terrain>>, focus: Focus) {
    let (Some(terrain), Some(at)) = (terrain, focus.planet()) else {
        return;
    };
    let globe = terrain.globe();
    let planet = (
        globe.seed(),
        globe.radius().to_bits(),
        globe.reserve().is_some(),
    );
    let known = &around.survey;
    let fresh = known.planet == planet && known.center.distance(at) <= f64::from(DRIFT);
    if around.task.is_none() && fresh {
        return;
    }
    let around = around.into_inner();
    if let Some(done) = around
        .task
        .as_mut()
        .and_then(|task| block_on(poll_once(task)))
    {
        (around.survey, around.task) = (done, None);
    } else if around.task.is_none() {
        let terrain = terrain.clone();
        let job = async move { survey(&terrain, at, AROUND + DRIFT) };
        around.task = Some(AsyncComputeTaskPool::get().spawn(job));
    }
}
