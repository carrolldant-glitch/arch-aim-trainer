use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct SessionStats {
    pub scenario: &'static str,
    pub duration_seconds: f32,
    pub score: u64,
    pub shots: u32,
    pub hits: u32,
    pub combo: u32,
    pub best_combo: u32,
    pub reaction_sum_ms: f64,
    pub reaction_samples: u32,
    pub tracking_on_target: f32,
    pub tracking_active: f32,
}

impl SessionStats {
    pub fn new(scenario: &'static str, duration_seconds: f32) -> Self {
        Self {
            scenario,
            duration_seconds,
            score: 0,
            shots: 0,
            hits: 0,
            combo: 0,
            best_combo: 0,
            reaction_sum_ms: 0.0,
            reaction_samples: 0,
            tracking_on_target: 0.0,
            tracking_active: 0.0,
        }
    }

    pub fn register_shot(&mut self) {
        self.shots += 1;
    }

    pub fn register_hit(&mut self, base_points: u64, reaction_ms: f64) {
        self.hits += 1;
        self.combo += 1;
        self.best_combo = self.best_combo.max(self.combo);
        let multiplier = 1.0 + (self.combo.min(20) as f64 * 0.025);
        self.score += (base_points as f64 * multiplier).round() as u64;
        self.reaction_sum_ms += reaction_ms;
        self.reaction_samples += 1;
    }

    pub fn register_miss(&mut self) {
        self.combo = 0;
    }

    pub fn register_tracking(&mut self, delta: f32, on_target: bool) {
        self.tracking_active += delta;
        if on_target {
            self.tracking_on_target += delta;
            self.score += (delta * 1000.0).round() as u64;
        } else {
            self.combo = 0;
        }
    }

    pub fn accuracy_percent(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.hits as f32 / self.shots as f32 * 100.0
        }
    }

    pub fn tracking_percent(&self) -> f32 {
        if self.tracking_active <= f32::EPSILON {
            0.0
        } else {
            self.tracking_on_target / self.tracking_active * 100.0
        }
    }

    pub fn average_reaction_ms(&self) -> f64 {
        if self.reaction_samples == 0 {
            0.0
        } else {
            self.reaction_sum_ms / self.reaction_samples as f64
        }
    }

    pub fn append_csv(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let needs_header = !path.exists() || fs::metadata(path)?.len() == 0;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        if needs_header {
            writeln!(
                file,
                "unix_time,scenario,duration_seconds,score,shots,hits,accuracy_percent,best_combo,average_reaction_ms,tracking_percent"
            )?;
        }
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        writeln!(
            file,
            "{unix_time},{},{:.1},{},{},{},{:.2},{},{:.2},{:.2}",
            self.scenario,
            self.duration_seconds,
            self.score,
            self.shots,
            self.hits,
            self.accuracy_percent(),
            self.best_combo,
            self.average_reaction_ms(),
            self.tracking_percent(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shot_accuracy_and_combo_are_consistent() {
        let mut stats = SessionStats::new("test", 60.0);
        stats.register_shot();
        stats.register_hit(100, 250.0);
        stats.register_shot();
        stats.register_miss();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.best_combo, 1);
        assert_eq!(stats.combo, 0);
        assert!((stats.accuracy_percent() - 50.0).abs() < f32::EPSILON);
        assert!((stats.average_reaction_ms() - 250.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tracking_percentage_uses_only_active_fire_time() {
        let mut stats = SessionStats::new("tracking", 60.0);
        stats.register_tracking(0.75, true);
        stats.register_tracking(0.25, false);
        assert!((stats.tracking_percent() - 75.0).abs() < f32::EPSILON);
    }
}
