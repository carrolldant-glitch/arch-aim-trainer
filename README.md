# Arch Aim Trainer

A fast, native 3D aim trainer built for Arch Linux and high-mobility hero shooters. It focuses on transferable mechanics—flick acquisition, sustained tracking, target switching, and constant-velocity projectile leading—without reading or interacting with any game process.

No game assets are included. This project is independent and is not affiliated with Valve, Deadlock, KovaaK's, or Aim Lab.

## Features

- Native OpenGL rendering with high-DPI, resizable, and fullscreen support
- Captured mouse input with configurable degrees-per-pixel sensitivity
- Horizontal FOV control from 60–140 degrees
- Four distinct 3D scenarios:
  - **Static Flick** — acquisition across varied depth and elevation
  - **Smooth Track** — reactive continuous tracking
  - **Target Switch** — controlled transfers among moving targets
  - **Projectile Lead** — finite-speed projectiles and an optional analytic lead guide
- WASD movement plus gravity-based jumping for airborne strafe-aim practice
- Score, accuracy, tracking percentage, reaction time, and combo statistics
- Persistent settings under the XDG configuration directory
- Append-only CSV session history under the XDG data directory
- No network access, telemetry, accounts, advertisements, or game integration

## Arch quick start

Install the native build requirements from the official repositories:

```bash
sudo pacman -S --needed rust pkgconf libx11 libxi libglvnd alsa-lib
```

Clone and run an optimized build:

```bash
git clone https://github.com/carrolldant-glitch/arch-aim-trainer.git
cd arch-aim-trainer
cargo run --release --locked
```

The first build downloads Rust crates from crates.io. Later locked builds can use Cargo's local cache.

## Controls

### Menu

| Keys | Action |
| --- | --- |
| `1`–`4`, `Up` / `Down` | Select scenario |
| `Enter` | Start |
| `Z` / `X` | Lower / raise sensitivity |
| `C` / `V` | Lower / raise horizontal FOV |
| `B` / `N` | Shorten / lengthen session |
| `G` / `H` | Shrink / enlarge targets |
| `J` / `K` | Lower / raise projectile speed |
| `L` | Toggle projectile lead guide |
| `F11` | Toggle fullscreen |
| `Q` | Quit |

### Session

| Keys | Action |
| --- | --- |
| Mouse | Aim |
| Mouse 1 | Fire; hold for Smooth Track |
| `WASD` | Move |
| `Space` | Jump; gravity returns you to the floor |
| `L` | Toggle lead guide in Projectile Lead |
| `R` | Restart scenario |
| `Esc` | Pause |
| `F11` | Toggle fullscreen |

## Settings and results

Settings are written only after an in-app adjustment:

```text
${XDG_CONFIG_HOME:-~/.config}/arch-aim-trainer/config.conf
```

Completed sessions are appended to:

```text
${XDG_DATA_HOME:-~/.local/share}/arch-aim-trainer/sessions.csv
```

See [docs/TUNING.md](docs/TUNING.md) for physical sensitivity matching and projectile calibration. An editable starting point is provided in [config.example.conf](config.example.conf).

## Development checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

The tests cover ray/sphere hits, finite segment collision, projectile interception, gravity and jumping, settings validation, scenario construction, and session statistics.

## Safety and competitive integrity

Arch Aim Trainer is an offline practice application. It does not inspect memory, inject code, create overlays on another application, synthesize game input, automate actions, or evade anti-cheat systems. Its lead guide exists only inside its own synthetic training arena.

## License

MIT. See [LICENSE](LICENSE).
