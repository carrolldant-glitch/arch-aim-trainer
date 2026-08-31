use macroquad::prelude::Vec3;

const EPSILON: f32 = 1.0e-6;

/// Return the nearest non-negative distance along a ray that intersects a sphere.
pub fn ray_sphere_hit(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let a = direction.length_squared();
    if a <= EPSILON || radius <= 0.0 {
        return None;
    }

    let offset = origin - center;
    let half_b = offset.dot(direction);
    let c = offset.length_squared() - radius * radius;
    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        return None;
    }

    let root = discriminant.sqrt();
    let near = (-half_b - root) / a;
    if near >= 0.0 {
        return Some(near);
    }

    let far = (-half_b + root) / a;
    (far >= 0.0).then_some(far)
}

/// Return whether a finite line segment intersects a sphere.
pub fn segment_sphere_hit(start: Vec3, end: Vec3, center: Vec3, radius: f32) -> bool {
    let segment = end - start;
    let length = segment.length();
    if length <= EPSILON {
        return start.distance_squared(center) <= radius * radius;
    }

    ray_sphere_hit(start, segment / length, center, radius)
        .is_some_and(|distance| distance <= length)
}

/// Solve the constant-velocity projectile intercept problem.
///
/// The returned time is the earliest positive intercept. Gravity and drag are
/// intentionally excluded so scenarios remain deterministic and explainable.
pub fn intercept_time(
    origin: Vec3,
    target_position: Vec3,
    target_velocity: Vec3,
    projectile_speed: f32,
) -> Option<f32> {
    if projectile_speed <= EPSILON {
        return None;
    }

    let relative = target_position - origin;
    let a = target_velocity.length_squared() - projectile_speed * projectile_speed;
    let b = 2.0 * relative.dot(target_velocity);
    let c = relative.length_squared();

    if a.abs() <= EPSILON {
        if b.abs() <= EPSILON {
            return None;
        }
        let time = -c / b;
        return (time > EPSILON).then_some(time);
    }

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let root = discriminant.sqrt();
    let first = (-b - root) / (2.0 * a);
    let second = (-b + root) / (2.0 * a);

    [first, second]
        .into_iter()
        .filter(|time| *time > EPSILON)
        .min_by(|left, right| left.total_cmp(right))
}

/// Convert a horizontal field of view in degrees to Macroquad's vertical FOV.
pub fn horizontal_to_vertical_fov(horizontal_degrees: f32, aspect: f32) -> f32 {
    let safe_aspect = aspect.max(0.1);
    let horizontal = horizontal_degrees.clamp(30.0, 170.0).to_radians();
    2.0 * ((horizontal * 0.5).tan() / safe_aspect).atan()
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::vec3;

    #[test]
    fn centered_ray_hits_near_surface() {
        let hit = ray_sphere_hit(Vec3::ZERO, vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, -10.0), 1.0);
        assert!((hit.expect("ray should hit") - 9.0).abs() < 1.0e-5);
    }

    #[test]
    fn ray_pointing_away_misses() {
        assert!(
            ray_sphere_hit(Vec3::ZERO, vec3(0.0, 0.0, 1.0), vec3(0.0, 0.0, -10.0), 1.0).is_none()
        );
    }

    #[test]
    fn segment_does_not_hit_past_its_endpoint() {
        assert!(!segment_sphere_hit(
            Vec3::ZERO,
            vec3(0.0, 0.0, -5.0),
            vec3(0.0, 0.0, -10.0),
            1.0,
        ));
    }

    #[test]
    fn stationary_target_intercept_is_distance_over_speed() {
        let time = intercept_time(Vec3::ZERO, vec3(0.0, 0.0, -20.0), Vec3::ZERO, 40.0)
            .expect("stationary target is reachable");
        assert!((time - 0.5).abs() < 1.0e-5);
    }

    #[test]
    fn lateral_target_intercept_matches_projectile_travel() {
        let origin = Vec3::ZERO;
        let target = vec3(0.0, 0.0, -20.0);
        let velocity = vec3(5.0, 0.0, 0.0);
        let speed = 30.0;
        let time = intercept_time(origin, target, velocity, speed).expect("target is reachable");
        let intercept = target + velocity * time;
        assert!((intercept.length() - speed * time).abs() < 1.0e-4);
    }
}
