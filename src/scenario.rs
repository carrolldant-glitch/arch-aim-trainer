use macroquad::prelude::*;

use crate::config::Settings;
use crate::math::{intercept_time, ray_sphere_hit, segment_sphere_hit};

const MIN_X: f32 = -9.0;
const MAX_X: f32 = 9.0;
const MIN_Y: f32 = 0.8;
const MAX_Y: f32 = 7.5;
const MIN_Z: f32 = -28.0;
const MAX_Z: f32 = -9.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioKind {
    StaticFlick,
    SmoothTrack,
    TargetSwitch,
    ProjectileLead,
}

impl ScenarioKind {
    pub const ALL: [Self; 4] = [
        Self::StaticFlick,
        Self::SmoothTrack,
        Self::TargetSwitch,
        Self::ProjectileLead,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::StaticFlick => "Static Flick",
            Self::SmoothTrack => "Smooth Track",
            Self::TargetSwitch => "Target Switch",
            Self::ProjectileLead => "Projectile Lead",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Self::StaticFlick => "Fast acquisition across varied depth and elevation",
            Self::SmoothTrack => "Sustain contact through reactive horizontal movement",
            Self::TargetSwitch => "Transfer cleanly between several moving threats",
            Self::ProjectileLead => "Lead mobile targets with finite projectile travel",
        }
    }

    pub fn instructions(self) -> &'static str {
        match self {
            Self::SmoothTrack => "Hold mouse 1 on the target",
            Self::ProjectileLead => "Click to fire  |  L toggles lead guide",
            _ => "Click targets as quickly and cleanly as possible",
        }
    }

    pub fn target_count(self) -> usize {
        match self {
            Self::StaticFlick => 6,
            Self::SmoothTrack => 1,
            Self::TargetSwitch => 5,
            Self::ProjectileLead => 3,
        }
    }

    pub fn is_tracking(self) -> bool {
        self == Self::SmoothTrack
    }

    pub fn uses_projectiles(self) -> bool {
        self == Self::ProjectileLead
    }
}

#[derive(Clone, Debug)]
pub struct Target {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
    pub spawned_at: f64,
    turn_at: f64,
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub previous: Vec3,
    pub position: Vec3,
    pub velocity: Vec3,
    pub age: f32,
    collided: bool,
}

#[derive(Clone, Debug)]
pub struct HitEvent {
    pub position: Vec3,
    pub reaction_ms: f64,
}

#[derive(Default)]
pub struct UpdateEvents {
    pub hits: Vec<HitEvent>,
    pub expired_projectiles: u32,
}

pub struct Arena {
    pub kind: ScenarioKind,
    pub targets: Vec<Target>,
    pub projectiles: Vec<Projectile>,
    target_scale: f32,
}

impl Arena {
    pub fn new(kind: ScenarioKind, settings: &Settings, now: f64) -> Self {
        let targets = (0..kind.target_count())
            .map(|ordinal| spawn_target(kind, settings.target_scale, now, ordinal))
            .collect();
        Self {
            kind,
            targets,
            projectiles: Vec::new(),
            target_scale: settings.target_scale,
        }
    }

    pub fn update(&mut self, delta: f32, now: f64) {
        for target in &mut self.targets {
            if target.velocity.length_squared() <= f32::EPSILON {
                continue;
            }

            if now >= target.turn_at {
                target.velocity = random_velocity(self.kind);
                target.turn_at = now + rand::gen_range(0.45, 1.15) as f64;
            }

            target.position += target.velocity * delta;
            bounce_axis(&mut target.position.x, &mut target.velocity.x, MIN_X, MAX_X);
            bounce_axis(&mut target.position.y, &mut target.velocity.y, MIN_Y, MAX_Y);
            bounce_axis(&mut target.position.z, &mut target.velocity.z, MIN_Z, MAX_Z);
        }
    }

    pub fn aimed_target(&self, origin: Vec3, direction: Vec3) -> Option<usize> {
        self.targets
            .iter()
            .enumerate()
            .filter_map(|(index, target)| {
                ray_sphere_hit(origin, direction, target.position, target.radius)
                    .map(|distance| (index, distance))
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(index, _)| index)
    }

    pub fn fire_hitscan(&mut self, origin: Vec3, direction: Vec3, now: f64) -> Option<HitEvent> {
        let index = self.aimed_target(origin, direction)?;
        let target = &self.targets[index];
        let event = HitEvent {
            position: target.position,
            reaction_ms: ((now - target.spawned_at) * 1000.0).max(0.0),
        };
        self.targets[index] = spawn_target(self.kind, self.target_scale, now, index);
        Some(event)
    }

    pub fn fire_projectile(&mut self, origin: Vec3, direction: Vec3, speed: f32) {
        self.projectiles.push(Projectile {
            previous: origin,
            position: origin,
            velocity: direction.normalize_or_zero() * speed,
            age: 0.0,
            collided: false,
        });
    }

    pub fn update_projectiles(&mut self, delta: f32, now: f64) -> UpdateEvents {
        let mut events = UpdateEvents::default();
        let mut hit_targets = vec![false; self.targets.len()];

        for projectile in &mut self.projectiles {
            projectile.previous = projectile.position;
            projectile.position += projectile.velocity * delta;
            projectile.age += delta;

            if projectile.age > 3.0 {
                events.expired_projectiles += 1;
                continue;
            }

            for (index, target) in self.targets.iter().enumerate() {
                if hit_targets[index] {
                    continue;
                }
                if segment_sphere_hit(
                    projectile.previous,
                    projectile.position,
                    target.position,
                    target.radius,
                ) {
                    projectile.collided = true;
                    hit_targets[index] = true;
                    events.hits.push(HitEvent {
                        position: target.position,
                        reaction_ms: ((now - target.spawned_at) * 1000.0).max(0.0),
                    });
                    break;
                }
            }
        }

        self.projectiles
            .retain(|projectile| projectile.age <= 3.0 && !projectile.collided);

        for (index, hit) in hit_targets.into_iter().enumerate() {
            if hit {
                self.targets[index] = spawn_target(self.kind, self.target_scale, now, index);
            }
        }

        events
    }

    pub fn lead_points(&self, origin: Vec3, projectile_speed: f32) -> Vec<Vec3> {
        self.targets
            .iter()
            .filter_map(|target| {
                intercept_time(origin, target.position, target.velocity, projectile_speed)
                    .filter(|time| *time <= 3.0)
                    .map(|time| target.position + target.velocity * time)
            })
            .collect()
    }
}

fn spawn_target(kind: ScenarioKind, scale: f32, now: f64, ordinal: usize) -> Target {
    let spread = ordinal as f32 / kind.target_count().max(1) as f32;
    let base_x = MIN_X + 1.0 + spread * (MAX_X - MIN_X - 2.0);
    let jitter = rand::gen_range(-1.2, 1.2);
    let radius = match kind {
        ScenarioKind::StaticFlick => 0.52,
        ScenarioKind::SmoothTrack => 0.72,
        ScenarioKind::TargetSwitch => 0.58,
        ScenarioKind::ProjectileLead => 0.68,
    } * scale;

    Target {
        position: vec3(
            (base_x + jitter).clamp(MIN_X, MAX_X),
            rand::gen_range(MIN_Y + radius, MAX_Y - radius),
            rand::gen_range(MIN_Z + 2.0, MAX_Z - 1.0),
        ),
        velocity: random_velocity(kind),
        radius,
        spawned_at: now,
        turn_at: now + rand::gen_range(0.45, 1.15) as f64,
    }
}

fn random_velocity(kind: ScenarioKind) -> Vec3 {
    match kind {
        ScenarioKind::StaticFlick => Vec3::ZERO,
        ScenarioKind::SmoothTrack => vec3(
            nonzero_random(-6.5, 6.5, 2.5),
            rand::gen_range(-2.2, 2.2),
            rand::gen_range(-0.8, 0.8),
        ),
        ScenarioKind::TargetSwitch => vec3(
            nonzero_random(-3.8, 3.8, 1.3),
            rand::gen_range(-1.5, 1.5),
            rand::gen_range(-0.6, 0.6),
        ),
        ScenarioKind::ProjectileLead => vec3(
            nonzero_random(-5.8, 5.8, 2.0),
            rand::gen_range(-1.8, 1.8),
            rand::gen_range(-0.7, 0.7),
        ),
    }
}

fn nonzero_random(minimum: f32, maximum: f32, minimum_magnitude: f32) -> f32 {
    let value = rand::gen_range(minimum, maximum);
    if value.abs() < minimum_magnitude {
        minimum_magnitude.copysign(if value == 0.0 { 1.0 } else { value })
    } else {
        value
    }
}

fn bounce_axis(position: &mut f32, velocity: &mut f32, minimum: f32, maximum: f32) {
    if *position < minimum {
        *position = minimum;
        *velocity = velocity.abs();
    } else if *position > maximum {
        *position = maximum;
        *velocity = -velocity.abs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scenario_has_targets() {
        let settings = Settings::default();
        for kind in ScenarioKind::ALL {
            let arena = Arena::new(kind, &settings, 0.0);
            assert_eq!(arena.targets.len(), kind.target_count());
        }
    }

    #[test]
    fn projectile_velocity_uses_configured_speed() {
        let settings = Settings::default();
        let mut arena = Arena::new(ScenarioKind::ProjectileLead, &settings, 0.0);
        arena.fire_projectile(Vec3::ZERO, vec3(0.0, 0.0, -1.0), 42.0);
        assert!((arena.projectiles[0].velocity.length() - 42.0).abs() < 1.0e-5);
    }
}
