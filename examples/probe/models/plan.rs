//! The list of shots: who, from where, in which pose.

use abstractions::companion::Expression;

/// Who is in the picture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Who {
    /// The robot alone.
    Robot,
    /// The companion alone.
    Buddy,
    /// Both, the companion at the robot's shoulder, for scale.
    Both,
}

/// Where the camera stands around the subject: the turn about the vertical in degrees from the
/// front, or above.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    /// Turned `0` degrees from the front: 0 front, 45 three-quarter front, 90 side, 135
    /// three-quarter back, 180 back.
    Around(f32),
    /// Straight above.
    Top,
}

/// What the robot is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    /// Standing.
    Idle,
    /// Walking.
    Walk,
    /// Running.
    Run,
    /// Crouched.
    Crouch,
    /// Gliding with the wings spread.
    Glide,
    /// Seated.
    Seat,
    /// Swimming at the surface.
    Swim,
    /// Holding a tool in one hand.
    Tool,
    /// Holding a blade two-handed.
    Blade,
    /// Holding a spear, ready to throw.
    Spear,
}

/// What the companion is doing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mood {
    /// Hovering, face neutral.
    Hover,
    /// Following at speed.
    Follow,
    /// Riding in the cockpit.
    Ride,
    /// Scanning, hands out.
    Scan,
    /// Showing an expression.
    Show(Expression),
}

/// One shot.
#[derive(Debug, Clone)]
pub struct Shot {
    /// File name without extension.
    pub name: String,
    /// Who is in it.
    pub who: Who,
    /// The view.
    pub view: View,
    /// Distance of the camera from what it looks at, meters.
    pub dist: f32,
    /// What the robot does.
    pub stance: Stance,
    /// What the companion does.
    pub mood: Mood,
}

/// Returns the shot of `who` named `name`.
fn shot(name: String, who: Who, (view, dist): (View, f32), (stance, mood): (Stance, Mood)) -> Shot {
    Shot {
        name,
        who,
        view,
        dist,
        stance,
        mood,
    }
}

/// Returns the file name of an expression.
fn face_name(e: Expression) -> &'static str {
    match e {
        Expression::Neutral => "neutral",
        Expression::Happy => "happy",
        Expression::Curious => "curious",
        Expression::Surprised => "surprised",
        Expression::Sleepy => "sleepy",
        Expression::Worried => "worried",
        Expression::Love => "love",
    }
}

/// Returns every shot: the turntables first, then the poses.
pub fn shots() -> Vec<Shot> {
    let views = [
        ("front", View::Around(0.0)),
        ("front34", View::Around(45.0)),
        ("side", View::Around(90.0)),
        ("back34", View::Around(135.0)),
        ("back", View::Around(180.0)),
        ("top", View::Top),
    ];
    let mut all = Vec::new();
    let rest = (Stance::Idle, Mood::Hover);
    for (who, label, near, far) in [
        (Who::Robot, "robot", 2.6, 8.0),
        (Who::Buddy, "buddy", 0.8, 3.0),
        (Who::Both, "pair", 3.0, 8.0),
    ] {
        for (name, view) in views {
            for (tag, dist) in [("near", near), ("far", far)] {
                let name = format!("{label}-{name}-{tag}");
                all.push(shot(name, who, (view, dist), rest));
            }
        }
    }
    let stances = [
        ("idle", Stance::Idle),
        ("walk", Stance::Walk),
        ("run", Stance::Run),
        ("crouch", Stance::Crouch),
        ("glide", Stance::Glide),
        ("seat", Stance::Seat),
        ("swim", Stance::Swim),
        ("tool", Stance::Tool),
        ("blade", Stance::Blade),
        ("spear", Stance::Spear),
    ];
    for (name, stance) in stances {
        for (tag, yaw) in [("front34", 45.0), ("side", 90.0)] {
            let name = format!("pose-{name}-{tag}");
            let view = (View::Around(yaw), 3.2);
            all.push(shot(name, Who::Robot, view, (stance, Mood::Hover)));
        }
    }
    let faces = Expression::ALL.map(|e| (face_name(e), Mood::Show(e)));
    let moods = [
        ("hover", Mood::Hover),
        ("follow", Mood::Follow),
        ("ride", Mood::Ride),
        ("scan", Mood::Scan),
    ]
    .into_iter()
    .chain(faces);
    for (name, mood) in moods {
        for (tag, yaw) in [("front", 0.0), ("front34", 45.0)] {
            let name = format!("buddy-{name}-{tag}");
            all.push(shot(name, Who::Buddy, (View::Around(yaw), 0.9), (Stance::Idle, mood)));
        }
    }
    all
}
