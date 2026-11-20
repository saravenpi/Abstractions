//! Tests of joints: every leg, neck, tail, wing and fin grows from inside the part it joins, and its
//! root is not much thinner than the limb that follows it, so no limb reads as glued to the outline.

use super::every;
use crate::life::body::{Body, Part, PartKind, ShapeKind, depth, limb, root};

/// Returns the next part of the limb part `index` of `body` starts: the shin under a thigh, the next
/// link of a neck or a tail.
fn next(body: &Body, index: usize) -> Option<&Part> {
    let kind = body.parts[index].kind;
    let follows =
        |p: &Part| p.kind == kind || (kind == PartKind::Thigh && p.kind == PartKind::Shin);
    body.parts
        .iter()
        .find(|p| p.parent == Some(index as u8) && follows(p))
}

/// Returns true when `part` is the root of a limb grown from `host`, a part with a volume.
fn grows(part: &Part, host: &Part) -> bool {
    let thin = matches!(host.shape, ShapeKind::Spike | ShapeKind::Fan);
    limb(part.kind) && host.kind != part.kind && !thin
}

/// Every limb root lies inside the volume of its parent, and is at least 0.6 times as wide as the
/// next part of its limb.
#[test]
fn limbs_grow_from_inside_the_body() {
    let mut roots = 0;
    every(|body, form, _| {
        for (i, part) in body.parts.iter().enumerate() {
            let Some(host) = part.parent.map(|p| &body.parts[usize::from(p)]) else {
                continue;
            };
            if !grows(part, host) {
                continue;
            }
            roots += 1;
            let inside = depth(host, part.socket);
            assert!(
                inside > 0.0,
                "{form:?} {:?} root {inside} m inside {:?}",
                part.kind,
                host.kind
            );
            if let Some(after) = next(body, i) {
                assert!(
                    root(part) >= 0.6 * root(after),
                    "{form:?} {:?} root {} thinner than {:?} {}",
                    part.kind,
                    root(part),
                    after.kind,
                    root(after)
                );
            }
        }
    });
    assert!(roots > 1000, "only {roots} limb roots");
}
