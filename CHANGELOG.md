# Changelog

## 0.1.1 - 2026-09-02

- Replace continuous vertical free flight with a grounded jump impulse,
  gravity, and floor landing.
- Keep horizontal movement speed independent from vertical velocity.
- Add regression tests for jump direction, gravity, landing, and mid-air jump
  rejection.

## 0.1.0 - 2026-08-31

- Add a native 3D Macroquad renderer and captured first-person camera.
- Add Static Flick, Smooth Track, Target Switch, and Projectile Lead scenarios.
- Add swept projectile collision and analytic constant-velocity lead prediction.
- Add sensitivity, FOV, duration, target-size, projectile-speed, movement, fullscreen, and lead-guide controls.
- Add score, accuracy, tracking, reaction-time, combo, and CSV history reporting.
- Add XDG-compliant persistent configuration.
- Add strict formatting, Clippy, unit-test, optimized-build, and Wayland launch verification.
