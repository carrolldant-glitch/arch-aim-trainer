mod config;
mod math;
mod scenario;
mod stats;

use macroquad::prelude::*;

use config::{Settings, results_path};
use math::horizontal_to_vertical_fov;
use scenario::{Arena, HitEvent, ScenarioKind};
use stats::SessionStats;

const BACKGROUND: Color = Color {
    r: 0.025,
    g: 0.035,
    b: 0.065,
    a: 1.0,
};
const PANEL: Color = Color {
    r: 0.055,
    g: 0.075,
    b: 0.12,
    a: 0.94,
};
const PANEL_SELECTED: Color = Color {
    r: 0.08,
    g: 0.16,
    b: 0.22,
    a: 0.98,
};
const TEXT: Color = Color {
    r: 0.90,
    g: 0.95,
    b: 1.0,
    a: 1.0,
};
const MUTED: Color = Color {
    r: 0.54,
    g: 0.63,
    b: 0.72,
    a: 1.0,
};
const ACCENT: Color = Color {
    r: 0.16,
    g: 0.86,
    b: 0.92,
    a: 1.0,
};
const SUCCESS: Color = Color {
    r: 0.30,
    g: 0.95,
    b: 0.54,
    a: 1.0,
};
const PLAYER_EYE_HEIGHT: f32 = 2.2;
const GRAVITY: f32 = 19.6;
const JUMP_VELOCITY: f32 = 7.0;
const GROUND_EPSILON: f32 = 0.001;

fn update_vertical_motion(height: &mut f32, velocity: &mut f32, delta: f32, jump_requested: bool) {
    let grounded = *height <= PLAYER_EYE_HEIGHT + GROUND_EPSILON && *velocity <= 0.0;
    if grounded {
        *height = PLAYER_EYE_HEIGHT;
        *velocity = 0.0;
        if jump_requested {
            *velocity = JUMP_VELOCITY;
        }
    }

    *velocity -= GRAVITY * delta;
    *height += *velocity * delta;

    if *height <= PLAYER_EYE_HEIGHT {
        *height = PLAYER_EYE_HEIGHT;
        *velocity = 0.0;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Arch Aim Trainer".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        fullscreen: false,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Menu,
    Playing,
    Paused,
    Results,
}

#[derive(Clone, Debug)]
struct CameraRig {
    position: Vec3,
    vertical_velocity: f32,
    yaw: f32,
    pitch: f32,
}

impl CameraRig {
    fn new() -> Self {
        Self {
            position: vec3(0.0, PLAYER_EYE_HEIGHT, 7.5),
            vertical_velocity: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
        .normalize()
    }

    fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize_or_zero()
    }

    fn update_look(&mut self, sensitivity: f32) {
        let local_delta = mouse_delta_position();
        let pixel_delta = vec2(
            local_delta.x * screen_width() * 0.5,
            local_delta.y * screen_height() * 0.5,
        );
        let radians_per_pixel = sensitivity.to_radians();
        self.yaw -= pixel_delta.x * radians_per_pixel;
        self.pitch += pixel_delta.y * radians_per_pixel;
        self.pitch = self
            .pitch
            .clamp(-88.0_f32.to_radians(), 88.0_f32.to_radians());
    }

    fn update_movement(&mut self, delta: f32, speed: f32) {
        let forward = vec3(self.forward().x, 0.0, self.forward().z).normalize_or_zero();
        let right = vec3(self.right().x, 0.0, self.right().z).normalize_or_zero();
        let mut movement = Vec3::ZERO;

        if is_key_down(KeyCode::W) {
            movement += forward;
        }
        if is_key_down(KeyCode::S) {
            movement -= forward;
        }
        if is_key_down(KeyCode::A) {
            movement -= right;
        }
        if is_key_down(KeyCode::D) {
            movement += right;
        }

        if speed > 0.0 && movement.length_squared() > 0.0 {
            self.position += movement.normalize() * speed * delta;
            self.position.x = self.position.x.clamp(-8.0, 8.0);
            self.position.z = self.position.z.clamp(2.0, 10.0);
        }

        update_vertical_motion(
            &mut self.position.y,
            &mut self.vertical_velocity,
            delta,
            is_key_pressed(KeyCode::Space),
        );
    }

    fn camera(&self, settings: &Settings) -> Camera3D {
        let aspect = (screen_width() / screen_height().max(1.0)).max(0.1);
        Camera3D {
            position: self.position,
            target: self.position + self.forward(),
            up: Vec3::Y,
            fovy: horizontal_to_vertical_fov(settings.horizontal_fov, aspect),
            z_near: 0.05,
            z_far: 100.0,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod movement_tests {
    use super::*;

    #[test]
    fn jump_changes_only_height_and_gravity_lands_on_the_floor() {
        let mut position = vec3(3.0, PLAYER_EYE_HEIGHT, -4.0);
        let mut velocity = 0.0;
        let mut peak = position.y;

        for frame in 0..240 {
            update_vertical_motion(&mut position.y, &mut velocity, 1.0 / 120.0, frame == 0);
            peak = peak.max(position.y);
        }

        assert!(peak > PLAYER_EYE_HEIGHT + 1.0);
        assert_eq!(position.x, 3.0);
        assert_eq!(position.z, -4.0);
        assert_eq!(position.y, PLAYER_EYE_HEIGHT);
        assert_eq!(velocity, 0.0);
    }

    #[test]
    fn jump_request_is_ignored_while_airborne() {
        let mut height = PLAYER_EYE_HEIGHT;
        let mut velocity = 0.0;

        update_vertical_motion(&mut height, &mut velocity, 1.0 / 60.0, true);
        let velocity_after_takeoff = velocity;
        update_vertical_motion(&mut height, &mut velocity, 1.0 / 60.0, true);

        assert!(height > PLAYER_EYE_HEIGHT);
        assert!(velocity < velocity_after_takeoff);
    }

    #[test]
    fn a_large_fall_step_cannot_tunnel_below_the_floor() {
        let mut height = PLAYER_EYE_HEIGHT + 0.1;
        let mut velocity = -100.0;

        update_vertical_motion(&mut height, &mut velocity, 0.05, false);

        assert_eq!(height, PLAYER_EYE_HEIGHT);
        assert_eq!(velocity, 0.0);
    }
}

#[derive(Clone, Debug)]
struct HitFlash {
    position: Vec3,
    until: f64,
}

struct Trainer {
    mode: Mode,
    selected: usize,
    settings: Settings,
    camera: CameraRig,
    arena: Arena,
    stats: SessionStats,
    elapsed: f32,
    hit_flash: Option<HitFlash>,
    miss_flash_until: f64,
    status: String,
    status_until: f64,
}

impl Trainer {
    fn new() -> Self {
        let settings = Settings::load();
        let kind = ScenarioKind::ALL[0];
        let now = get_time();
        Self {
            mode: Mode::Menu,
            selected: 0,
            camera: CameraRig::new(),
            arena: Arena::new(kind, &settings, now),
            stats: SessionStats::new(kind.name(), settings.session_seconds),
            elapsed: 0.0,
            hit_flash: None,
            miss_flash_until: 0.0,
            status: String::new(),
            status_until: 0.0,
            settings,
        }
    }

    fn kind(&self) -> ScenarioKind {
        ScenarioKind::ALL[self.selected]
    }

    fn start_session(&mut self) {
        let now = get_time();
        self.camera.reset();
        self.arena = Arena::new(self.kind(), &self.settings, now);
        self.stats = SessionStats::new(self.kind().name(), self.settings.session_seconds);
        self.elapsed = 0.0;
        self.hit_flash = None;
        self.miss_flash_until = 0.0;
        self.mode = Mode::Playing;
        set_cursor_grab(true);
        show_mouse(false);
    }

    fn return_to_menu(&mut self) {
        self.mode = Mode::Menu;
        set_cursor_grab(false);
        show_mouse(true);
    }

    fn finish_session(&mut self) {
        self.mode = Mode::Results;
        self.stats.duration_seconds = self.elapsed;
        set_cursor_grab(false);
        show_mouse(true);

        self.status = match results_path() {
            Some(path) => match self.stats.append_csv(&path) {
                Ok(()) => format!("Session saved to {}", path.display()),
                Err(error) => format!("Could not save session: {error}"),
            },
            None => "Session complete; HOME is unset, so history was not saved".to_owned(),
        };
        self.status_until = get_time() + 15.0;
    }

    fn save_settings(&mut self) {
        match self.settings.save() {
            Ok(path) => self.set_status(format!("Settings saved to {}", path.display())),
            Err(error) => self.set_status(format!("Could not save settings: {error}")),
        }
    }

    fn set_status(&mut self, message: String) {
        self.status = message;
        self.status_until = get_time() + 4.0;
    }

    fn toggle_fullscreen(&mut self) {
        self.settings.fullscreen = !self.settings.fullscreen;
        set_fullscreen(self.settings.fullscreen);
        self.save_settings();
    }

    fn handle_menu_input(&mut self) -> bool {
        if is_key_pressed(KeyCode::Q) {
            return true;
        }

        if is_key_pressed(KeyCode::Key1) {
            self.selected = 0;
        }
        if is_key_pressed(KeyCode::Key2) {
            self.selected = 1;
        }
        if is_key_pressed(KeyCode::Key3) {
            self.selected = 2;
        }
        if is_key_pressed(KeyCode::Key4) {
            self.selected = 3;
        }
        if is_key_pressed(KeyCode::Up) {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(ScenarioKind::ALL.len() - 1);
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected = (self.selected + 1) % ScenarioKind::ALL.len();
        }

        let mut changed = false;
        if is_key_pressed(KeyCode::Z) {
            self.settings.sensitivity = (self.settings.sensitivity - 0.005).max(0.005);
            changed = true;
        }
        if is_key_pressed(KeyCode::X) {
            self.settings.sensitivity = (self.settings.sensitivity + 0.005).min(1.0);
            changed = true;
        }
        if is_key_pressed(KeyCode::C) {
            self.settings.horizontal_fov = (self.settings.horizontal_fov - 1.0).max(60.0);
            changed = true;
        }
        if is_key_pressed(KeyCode::V) {
            self.settings.horizontal_fov = (self.settings.horizontal_fov + 1.0).min(140.0);
            changed = true;
        }
        if is_key_pressed(KeyCode::B) {
            self.settings.session_seconds = (self.settings.session_seconds - 15.0).max(15.0);
            changed = true;
        }
        if is_key_pressed(KeyCode::N) {
            self.settings.session_seconds = (self.settings.session_seconds + 15.0).min(600.0);
            changed = true;
        }
        if is_key_pressed(KeyCode::G) {
            self.settings.target_scale = (self.settings.target_scale - 0.05).max(0.4);
            changed = true;
        }
        if is_key_pressed(KeyCode::H) {
            self.settings.target_scale = (self.settings.target_scale + 0.05).min(2.5);
            changed = true;
        }
        if is_key_pressed(KeyCode::J) {
            self.settings.projectile_speed = (self.settings.projectile_speed - 1.0).max(8.0);
            changed = true;
        }
        if is_key_pressed(KeyCode::K) {
            self.settings.projectile_speed = (self.settings.projectile_speed + 1.0).min(120.0);
            changed = true;
        }
        if is_key_pressed(KeyCode::L) {
            self.settings.lead_guide = !self.settings.lead_guide;
            changed = true;
        }

        if changed {
            self.save_settings();
        }
        if is_key_pressed(KeyCode::Enter) {
            self.start_session();
        }
        false
    }

    fn handle_playing_input(&mut self, delta: f32) {
        if is_key_pressed(KeyCode::Escape) {
            self.mode = Mode::Paused;
            set_cursor_grab(false);
            show_mouse(true);
            return;
        }
        if is_key_pressed(KeyCode::R) {
            self.start_session();
            return;
        }
        if is_key_pressed(KeyCode::L) && self.kind().uses_projectiles() {
            self.settings.lead_guide = !self.settings.lead_guide;
            self.save_settings();
        }

        self.camera.update_look(self.settings.sensitivity);
        self.camera
            .update_movement(delta, self.settings.movement_speed);

        let now = get_time();
        self.arena.update(delta, now);
        let origin = self.camera.position;
        let direction = self.camera.forward();

        if self.kind().is_tracking() {
            if is_mouse_button_down(MouseButton::Left) {
                let on_target = self.arena.aimed_target(origin, direction).is_some();
                self.stats.register_tracking(delta, on_target);
                if on_target {
                    self.stats.combo = self.stats.combo.saturating_add(1);
                    self.stats.best_combo = self.stats.best_combo.max(self.stats.combo);
                    self.hit_flash =
                        self.arena
                            .aimed_target(origin, direction)
                            .map(|index| HitFlash {
                                position: self.arena.targets[index].position,
                                until: now + 0.04,
                            });
                }
            }
        } else if is_mouse_button_pressed(MouseButton::Left) {
            self.stats.register_shot();
            if self.kind().uses_projectiles() {
                self.arena
                    .fire_projectile(origin, direction, self.settings.projectile_speed);
            } else if let Some(hit) = self.arena.fire_hitscan(origin, direction, now) {
                self.register_hit(hit);
            } else {
                self.stats.register_miss();
                self.miss_flash_until = now + 0.12;
            }
        }

        if self.kind().uses_projectiles() {
            let events = self.arena.update_projectiles(delta, now);
            for hit in events.hits {
                self.register_hit(hit);
            }
            if events.expired_projectiles > 0 {
                self.stats.register_miss();
            }
        }

        self.elapsed += delta;
        if self.elapsed >= self.settings.session_seconds {
            self.finish_session();
        }
    }

    fn register_hit(&mut self, hit: HitEvent) {
        let speed_bonus = (45_000.0 / (hit.reaction_ms + 150.0)).clamp(0.0, 350.0) as u64;
        let base = match self.kind() {
            ScenarioKind::StaticFlick => 150,
            ScenarioKind::TargetSwitch => 180,
            ScenarioKind::ProjectileLead => 240,
            ScenarioKind::SmoothTrack => 0,
        };
        self.stats.register_hit(base + speed_bonus, hit.reaction_ms);
        self.hit_flash = Some(HitFlash {
            position: hit.position,
            until: get_time() + 0.16,
        });
    }

    fn handle_paused_input(&mut self) {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            self.mode = Mode::Playing;
            set_cursor_grab(true);
            show_mouse(false);
        } else if is_key_pressed(KeyCode::R) {
            self.start_session();
        } else if is_key_pressed(KeyCode::M) {
            self.return_to_menu();
        }
    }

    fn handle_results_input(&mut self) {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::R) {
            self.start_session();
        } else if is_key_pressed(KeyCode::M) || is_key_pressed(KeyCode::Escape) {
            self.return_to_menu();
        }
    }

    fn draw(&self) {
        match self.mode {
            Mode::Menu => self.draw_menu(),
            Mode::Playing => self.draw_world(false),
            Mode::Paused => {
                self.draw_world(true);
                self.draw_pause_overlay();
            }
            Mode::Results => self.draw_results(),
        }
    }

    fn draw_menu(&self) {
        clear_background(BACKGROUND);
        draw_rectangle(0.0, 0.0, screen_width(), 6.0, ACCENT);
        draw_text("ARCH AIM TRAINER", 42.0, 64.0, 38.0, TEXT);
        draw_text(
            "Native Linux practice for flicks, tracking, switches, and projectile leading",
            44.0,
            92.0,
            20.0,
            MUTED,
        );

        let margin = 42.0;
        let gap = 18.0;
        let card_width = (screen_width() - margin * 2.0 - gap) * 0.5;
        let card_height = 116.0;
        for (index, kind) in ScenarioKind::ALL.iter().copied().enumerate() {
            let column = (index % 2) as f32;
            let row = (index / 2) as f32;
            let x = margin + column * (card_width + gap);
            let y = 126.0 + row * (card_height + gap);
            let selected = index == self.selected;
            draw_rectangle(
                x,
                y,
                card_width,
                card_height,
                if selected { PANEL_SELECTED } else { PANEL },
            );
            draw_rectangle_lines(
                x,
                y,
                card_width,
                card_height,
                if selected { 2.5 } else { 1.0 },
                if selected {
                    ACCENT
                } else {
                    Color::new(0.16, 0.22, 0.30, 1.0)
                },
            );
            draw_text(format!("{}", index + 1), x + 18.0, y + 31.0, 20.0, ACCENT);
            draw_text(kind.name(), x + 50.0, y + 34.0, 25.0, TEXT);
            draw_text(kind.subtitle(), x + 18.0, y + 69.0, 17.0, MUTED);
            draw_text(kind.instructions(), x + 18.0, y + 96.0, 15.0, MUTED);
        }

        let settings_y = 402.0;
        draw_rectangle(
            margin,
            settings_y,
            screen_width() - margin * 2.0,
            194.0,
            PANEL,
        );
        draw_text("TUNING", margin + 20.0, settings_y + 33.0, 21.0, ACCENT);
        draw_setting(
            margin + 20.0,
            settings_y + 68.0,
            "Z / X",
            "Sensitivity",
            &format!("{:.3} deg/px", self.settings.sensitivity),
        );
        draw_setting(
            margin + 20.0,
            settings_y + 103.0,
            "C / V",
            "Horizontal FOV",
            &format!("{:.0} deg", self.settings.horizontal_fov),
        );
        draw_setting(
            margin + 20.0,
            settings_y + 138.0,
            "B / N",
            "Session",
            &format!("{:.0} sec", self.settings.session_seconds),
        );
        draw_setting(
            margin + 20.0,
            settings_y + 173.0,
            "G / H",
            "Target scale",
            &format!("{:.2}x", self.settings.target_scale),
        );

        let right = screen_width() * 0.53;
        draw_setting(
            right,
            settings_y + 68.0,
            "J / K",
            "Projectile speed",
            &format!("{:.0} u/s", self.settings.projectile_speed),
        );
        draw_setting(
            right,
            settings_y + 103.0,
            "L",
            "Lead guide",
            if self.settings.lead_guide {
                "on"
            } else {
                "off"
            },
        );
        draw_setting(
            right,
            settings_y + 138.0,
            "F11",
            "Fullscreen",
            if self.settings.fullscreen {
                "on"
            } else {
                "off"
            },
        );
        draw_setting(
            right,
            settings_y + 173.0,
            "WASD",
            "Movement",
            "Space to jump",
        );

        draw_text(
            "ENTER  START",
            margin,
            screen_height() - 48.0,
            24.0,
            SUCCESS,
        );
        draw_text(
            "UP/DOWN  SELECT    Q  QUIT",
            margin + 190.0,
            screen_height() - 48.0,
            18.0,
            MUTED,
        );
        if get_time() < self.status_until {
            draw_text(&self.status, margin, screen_height() - 18.0, 15.0, MUTED);
        }
    }

    fn draw_world(&self, dimmed: bool) {
        clear_background(BACKGROUND);
        let origin = self.camera.position;
        let direction = self.camera.forward();
        let aimed = self.arena.aimed_target(origin, direction);

        set_camera(&self.camera.camera(&self.settings));
        draw_grid(
            48,
            1.0,
            Color::new(0.10, 0.16, 0.23, 1.0),
            Color::new(0.055, 0.085, 0.13, 1.0),
        );
        draw_cube_wires(
            vec3(0.0, 4.0, -18.5),
            vec3(20.0, 8.0, 20.0),
            Color::new(0.10, 0.22, 0.31, 0.75),
        );

        let base_color = scenario_color(self.kind());
        for (index, target) in self.arena.targets.iter().enumerate() {
            let color = if aimed == Some(index) {
                WHITE
            } else {
                base_color
            };
            draw_sphere(target.position, target.radius, None, color);
            draw_sphere_wires(
                target.position,
                target.radius * 1.08,
                None,
                Color::new(color.r, color.g, color.b, 0.85),
            );
        }

        for projectile in &self.arena.projectiles {
            draw_sphere(
                projectile.position,
                0.10,
                None,
                Color::new(1.0, 0.90, 0.25, 1.0),
            );
            draw_line_3d(
                projectile.previous,
                projectile.position,
                Color::new(1.0, 0.62, 0.16, 0.9),
            );
        }

        if self.kind().uses_projectiles() && self.settings.lead_guide {
            for point in self
                .arena
                .lead_points(self.camera.position, self.settings.projectile_speed)
            {
                draw_sphere_wires(point, 0.22, None, Color::new(0.30, 1.0, 0.72, 0.82));
                draw_line_3d(point - Vec3::Y * 0.35, point + Vec3::Y * 0.35, SUCCESS);
            }
        }

        if let Some(flash) = &self.hit_flash {
            if get_time() <= flash.until {
                let pulse = 0.22 + ((flash.until - get_time()) as f32 * 2.0);
                draw_sphere_wires(flash.position, pulse, None, WHITE);
            }
        }

        set_default_camera();
        self.draw_hud(aimed.is_some());
        if dimmed {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, 0.55),
            );
        }
    }

    fn draw_hud(&self, on_target: bool) {
        let remaining = (self.settings.session_seconds - self.elapsed).max(0.0);
        draw_rectangle(
            18.0,
            18.0,
            286.0,
            104.0,
            Color::new(0.02, 0.03, 0.055, 0.86),
        );
        draw_text(self.kind().name(), 32.0, 47.0, 23.0, TEXT);
        draw_text(
            format!("TIME  {:05.1}", remaining),
            32.0,
            76.0,
            20.0,
            ACCENT,
        );
        draw_text(
            format!("SCORE  {}", self.stats.score),
            32.0,
            104.0,
            20.0,
            TEXT,
        );

        let right_x = screen_width() - 268.0;
        draw_rectangle(
            right_x,
            18.0,
            250.0,
            104.0,
            Color::new(0.02, 0.03, 0.055, 0.86),
        );
        if self.kind().is_tracking() {
            draw_text(
                format!("ON TARGET  {:05.1}%", self.stats.tracking_percent()),
                right_x + 16.0,
                49.0,
                20.0,
                TEXT,
            );
        } else {
            draw_text(
                format!("ACCURACY  {:05.1}%", self.stats.accuracy_percent()),
                right_x + 16.0,
                49.0,
                20.0,
                TEXT,
            );
            draw_text(
                format!("HITS  {} / {}", self.stats.hits, self.stats.shots),
                right_x + 16.0,
                76.0,
                18.0,
                MUTED,
            );
        }
        draw_text(
            format!(
                "COMBO  {}   BEST  {}",
                self.stats.combo, self.stats.best_combo
            ),
            right_x + 16.0,
            103.0,
            18.0,
            MUTED,
        );

        let center = vec2(screen_width() * 0.5, screen_height() * 0.5);
        let crosshair = if get_time() < self.miss_flash_until {
            RED
        } else if on_target {
            SUCCESS
        } else {
            WHITE
        };
        draw_line(
            center.x - 13.0,
            center.y,
            center.x - 4.0,
            center.y,
            2.0,
            crosshair,
        );
        draw_line(
            center.x + 4.0,
            center.y,
            center.x + 13.0,
            center.y,
            2.0,
            crosshair,
        );
        draw_line(
            center.x,
            center.y - 13.0,
            center.x,
            center.y - 4.0,
            2.0,
            crosshair,
        );
        draw_line(
            center.x,
            center.y + 4.0,
            center.x,
            center.y + 13.0,
            2.0,
            crosshair,
        );
        draw_circle_lines(center.x, center.y, 2.2, 1.4, crosshair);

        if self
            .hit_flash
            .as_ref()
            .is_some_and(|flash| get_time() <= flash.until)
        {
            draw_line(
                center.x - 10.0,
                center.y - 10.0,
                center.x - 4.0,
                center.y - 4.0,
                2.0,
                WHITE,
            );
            draw_line(
                center.x + 10.0,
                center.y - 10.0,
                center.x + 4.0,
                center.y - 4.0,
                2.0,
                WHITE,
            );
            draw_line(
                center.x - 10.0,
                center.y + 10.0,
                center.x - 4.0,
                center.y + 4.0,
                2.0,
                WHITE,
            );
            draw_line(
                center.x + 10.0,
                center.y + 10.0,
                center.x + 4.0,
                center.y + 4.0,
                2.0,
                WHITE,
            );
        }

        draw_text(
            "ESC pause   R restart   F11 fullscreen",
            18.0,
            screen_height() - 18.0,
            15.0,
            MUTED,
        );
        draw_text(
            format!("{} FPS", get_fps()),
            screen_width() - 70.0,
            screen_height() - 18.0,
            15.0,
            MUTED,
        );
    }

    fn draw_pause_overlay(&self) {
        let width = 430.0;
        let height = 220.0;
        let x = (screen_width() - width) * 0.5;
        let y = (screen_height() - height) * 0.5;
        draw_rectangle(x, y, width, height, PANEL);
        draw_rectangle_lines(x, y, width, height, 2.0, ACCENT);
        draw_centered("PAUSED", y + 54.0, 34.0, TEXT);
        draw_centered("ENTER / ESC  resume", y + 104.0, 20.0, MUTED);
        draw_centered("R  restart session", y + 139.0, 20.0, MUTED);
        draw_centered("M  return to menu", y + 174.0, 20.0, MUTED);
    }

    fn draw_results(&self) {
        clear_background(BACKGROUND);
        draw_rectangle(0.0, 0.0, screen_width(), 6.0, SUCCESS);
        draw_centered("SESSION COMPLETE", 82.0, 38.0, TEXT);
        draw_centered(self.kind().name(), 116.0, 21.0, ACCENT);

        let width = screen_width().min(760.0);
        let x = (screen_width() - width) * 0.5;
        draw_rectangle(x, 150.0, width, 360.0, PANEL);
        draw_metric(
            x + 42.0,
            205.0,
            "SCORE",
            &self.stats.score.to_string(),
            SUCCESS,
        );
        draw_metric(
            x + width * 0.52,
            205.0,
            if self.kind().is_tracking() {
                "ON TARGET"
            } else {
                "ACCURACY"
            },
            &format!(
                "{:.1}%",
                if self.kind().is_tracking() {
                    self.stats.tracking_percent()
                } else {
                    self.stats.accuracy_percent()
                }
            ),
            ACCENT,
        );
        draw_metric(x + 42.0, 310.0, "HITS", &self.stats.hits.to_string(), TEXT);
        draw_metric(
            x + width * 0.52,
            310.0,
            "BEST COMBO",
            &self.stats.best_combo.to_string(),
            TEXT,
        );
        let reaction_value = if self.stats.reaction_samples > 0 {
            format!("{:.0} ms", self.stats.average_reaction_ms())
        } else {
            "n/a".to_owned()
        };
        draw_metric(x + 42.0, 415.0, "AVG REACTION", &reaction_value, TEXT);
        draw_metric(
            x + width * 0.52,
            415.0,
            "DURATION",
            &format!("{:.1} sec", self.stats.duration_seconds),
            TEXT,
        );

        draw_centered("ENTER  run again     M / ESC  menu", 558.0, 22.0, MUTED);
        if get_time() < self.status_until {
            draw_centered(&self.status, 602.0, 16.0, MUTED);
        }
    }
}

fn scenario_color(kind: ScenarioKind) -> Color {
    match kind {
        ScenarioKind::StaticFlick => Color::new(0.14, 0.82, 0.96, 1.0),
        ScenarioKind::SmoothTrack => Color::new(0.32, 0.95, 0.50, 1.0),
        ScenarioKind::TargetSwitch => Color::new(1.0, 0.55, 0.20, 1.0),
        ScenarioKind::ProjectileLead => Color::new(0.94, 0.27, 0.75, 1.0),
    }
}

fn draw_setting(x: f32, y: f32, key: &str, label: &str, value: &str) {
    draw_text(key, x, y, 16.0, ACCENT);
    draw_text(label, x + 72.0, y, 17.0, TEXT);
    draw_text(value, x + 220.0, y, 17.0, MUTED);
}

fn draw_centered(text: &str, y: f32, size: f32, color: Color) {
    let dimensions = measure_text(text, None, size as u16, 1.0);
    draw_text(
        text,
        (screen_width() - dimensions.width) * 0.5,
        y,
        size,
        color,
    );
}

fn draw_metric(x: f32, y: f32, label: &str, value: &str, color: Color) {
    draw_text(label, x, y, 17.0, MUTED);
    draw_text(value, x, y + 42.0, 35.0, color);
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut trainer = Trainer::new();
    if trainer.settings.fullscreen {
        set_fullscreen(true);
    }

    loop {
        let delta = get_frame_time().min(0.05);
        if is_key_pressed(KeyCode::F11) {
            trainer.toggle_fullscreen();
        }

        let quit = match trainer.mode {
            Mode::Menu => trainer.handle_menu_input(),
            Mode::Playing => {
                trainer.handle_playing_input(delta);
                false
            }
            Mode::Paused => {
                trainer.handle_paused_input();
                false
            }
            Mode::Results => {
                trainer.handle_results_input();
                false
            }
        };
        if quit {
            break;
        }

        trainer.draw();
        next_frame().await;
    }
}
