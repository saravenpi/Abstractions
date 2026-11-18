//! One row of the flyover table: what a whole flight showed.

use super::STEP;
use super::probe::{KINDS, Sample};
use super::rig::STEPS;

/// What a whole flyover showed.
#[derive(Debug, Clone, Copy)]
pub struct Row {
    /// Distance in metres at which the fog hides everything, at the end of the flight.
    pub fog: f64,
    /// Nearest unsettled ground ahead over the flight, in metres.
    pub ground: f64,
    /// Nearest unsettled ground in any direction.
    pub around: f64,
    /// Nearest ground ahead that only lacked fine detail.
    pub fine: f64,
    /// Share of the frames with unsettled ground ahead within the fog.
    pub late: f64,
    /// Things that appeared within the fog per second, by kind.
    pub pops: [f64; KINDS],
    /// Of those, the ones that faded in, per second.
    pub faded: f64,
    /// Most requests waiting in one frame.
    pub queue: usize,
    /// Longest wait of a request in seconds.
    pub age: f64,
    /// Mean processor milliseconds of each step of the chain per frame.
    pub steps: [f64; STEPS.len()],
    /// Processor milliseconds of the streaming chain in its worst frame.
    pub worst: f64,
}

impl Default for Row {
    /// Returns a row with nothing flown: no gap seen yet, at any distance.
    fn default() -> Self {
        Self {
            fog: 0.0,
            ground: f64::INFINITY,
            around: f64::INFINITY,
            fine: f64::INFINITY,
            late: 0.0,
            pops: [0.0; KINDS],
            faded: 0.0,
            queue: 0,
            age: 0.0,
            steps: [0.0; STEPS.len()],
            worst: 0.0,
        }
    }
}

impl Row {
    /// Adds the `sample` of one frame of a flight of `seconds`, seen through fog that ends
    /// `fog` metres away, whose steps took `millis`.
    pub fn add(&mut self, sample: &Sample, (fog, seconds): (f64, f64), millis: &[f64]) {
        let frames = seconds / STEP;
        self.fog = fog;
        self.ground = self.ground.min(sample.ground);
        self.around = self.around.min(sample.around);
        self.fine = self.fine.min(sample.fine);
        self.late += f64::from(u8::from(sample.ground < fog)) / frames;
        for (sum, pops) in self.pops.iter_mut().zip(sample.pops) {
            *sum += f64::from(pops) / seconds;
        }
        self.faded += f64::from(sample.faded) / seconds;
        self.queue = self.queue.max(sample.queue);
        self.age = self.age.max(f64::from(sample.age) * STEP);
        for (sum, step) in self.steps.iter_mut().zip(millis) {
            *sum += step / frames;
        }
        self.worst = self.worst.max(millis.iter().sum());
    }

    /// Returns the things that appeared at once within the fog per second, all kinds together.
    pub fn popped(&self) -> f64 {
        self.pops.iter().sum::<f64>() - self.faded
    }

    /// Returns the mean processor milliseconds of the streaming chain per frame.
    pub fn mean(&self) -> f64 {
        self.steps.iter().sum()
    }

    /// Returns the row as text: what was unsettled, what popped, what waited and what it cost.
    pub fn text(&self) -> String {
        let steps = STEPS.iter().zip(self.steps);
        let steps: Vec<String> = steps.map(|(name, ms)| format!("{name} {ms:.2}")).collect();
        let steps = steps.join(" ");
        format!(
            "fog {:>5.0} m | unsettled ahead {:>5.0} m any {:>5.0} m fine {:>5.0} m late {:>3.0} % | \
             in fog per s: node {:>5.1} chunk {:>5.1} level {:>5.1} cover {:>5.1} faded {:>5.1} \
             popped {:>5.1} | queue {:>3} age {:>4.1} s | main {:>5.2} ms worst {:>6.2} ms | {steps}",
            self.fog,
            self.ground,
            self.around,
            self.fine,
            self.late * 100.0,
            self.pops[0],
            self.pops[1],
            self.pops[2],
            self.pops[3],
            self.faded,
            self.popped(),
            self.queue,
            self.age,
            self.mean(),
            self.worst
        )
    }
}
