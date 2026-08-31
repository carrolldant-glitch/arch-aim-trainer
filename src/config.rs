use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Settings {
    pub sensitivity: f32,
    pub horizontal_fov: f32,
    pub session_seconds: f32,
    pub target_scale: f32,
    pub projectile_speed: f32,
    pub movement_speed: f32,
    pub fullscreen: bool,
    pub lead_guide: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: 0.085,
            horizontal_fov: 103.0,
            session_seconds: 60.0,
            target_scale: 1.0,
            projectile_speed: 38.0,
            movement_speed: 6.5,
            fullscreen: false,
            lead_guide: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let mut settings = Self::default();
        let Some(path) = config_path() else {
            return settings;
        };
        let Ok(contents) = fs::read_to_string(path) else {
            return settings;
        };

        settings.apply_text(&contents);
        settings.sanitize();
        settings
    }

    pub fn save(&self) -> io::Result<PathBuf> {
        let path = config_path().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "HOME and XDG_CONFIG_HOME are unset",
            )
        })?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, self.to_text())?;
        Ok(path)
    }

    fn apply_text(&mut self, contents: &str) {
        for raw_line in contents.lines() {
            let line = raw_line.split('#').next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                "sensitivity" => parse_f32(value, &mut self.sensitivity),
                "horizontal_fov" => parse_f32(value, &mut self.horizontal_fov),
                "session_seconds" => parse_f32(value, &mut self.session_seconds),
                "target_scale" => parse_f32(value, &mut self.target_scale),
                "projectile_speed" => parse_f32(value, &mut self.projectile_speed),
                "movement_speed" => parse_f32(value, &mut self.movement_speed),
                "fullscreen" => parse_bool(value, &mut self.fullscreen),
                "lead_guide" => parse_bool(value, &mut self.lead_guide),
                _ => {}
            }
        }
    }

    fn sanitize(&mut self) {
        self.sensitivity = self.sensitivity.clamp(0.005, 1.0);
        self.horizontal_fov = self.horizontal_fov.clamp(60.0, 140.0);
        self.session_seconds = self.session_seconds.clamp(15.0, 600.0);
        self.target_scale = self.target_scale.clamp(0.4, 2.5);
        self.projectile_speed = self.projectile_speed.clamp(8.0, 120.0);
        self.movement_speed = self.movement_speed.clamp(0.0, 20.0);
    }

    fn to_text(&self) -> String {
        format!(
            concat!(
                "# Arch Aim Trainer settings\n",
                "# sensitivity is degrees per mouse pixel\n",
                "sensitivity={:.4}\n",
                "horizontal_fov={:.1}\n",
                "session_seconds={:.0}\n",
                "target_scale={:.2}\n",
                "projectile_speed={:.1}\n",
                "movement_speed={:.1}\n",
                "fullscreen={}\n",
                "lead_guide={}\n",
            ),
            self.sensitivity,
            self.horizontal_fov,
            self.session_seconds,
            self.target_scale,
            self.projectile_speed,
            self.movement_speed,
            self.fullscreen,
            self.lead_guide,
        )
    }
}

fn parse_f32(value: &str, destination: &mut f32) {
    if let Ok(parsed) = value.parse() {
        *destination = parsed;
    }
}

fn parse_bool(value: &str, destination: &mut bool) {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => *destination = true,
        "false" | "no" | "0" | "off" => *destination = false,
        _ => {}
    }
}

fn config_path() -> Option<PathBuf> {
    if let Some(root) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(root).join("arch-aim-trainer/config.conf"));
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/arch-aim-trainer/config.conf"))
}

pub fn results_path() -> Option<PathBuf> {
    if let Some(root) = env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(root).join("arch-aim-trainer/sessions.csv"));
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".local/share/arch-aim-trainer/sessions.csv"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_known_values_and_ignores_unknown_ones() {
        let mut settings = Settings::default();
        settings.apply_text("sensitivity=0.12\nhorizontal_fov=110\nlead_guide=off\nunknown=42\n");
        settings.sanitize();
        assert!((settings.sensitivity - 0.12).abs() < f32::EPSILON);
        assert!((settings.horizontal_fov - 110.0).abs() < f32::EPSILON);
        assert!(!settings.lead_guide);
    }

    #[test]
    fn unsafe_values_are_clamped() {
        let mut settings = Settings::default();
        settings.apply_text("sensitivity=-5\nhorizontal_fov=900\nsession_seconds=1\n");
        settings.sanitize();
        assert_eq!(settings.sensitivity, 0.005);
        assert_eq!(settings.horizontal_fov, 140.0);
        assert_eq!(settings.session_seconds, 15.0);
    }
}
