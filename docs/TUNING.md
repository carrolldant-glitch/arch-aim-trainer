# Tuning for your setup

## Match physical sensitivity

The trainer stores sensitivity as degrees of camera rotation per mouse pixel reported by the window system. Game sensitivity scales are not interchangeable, so match the physical distance of one complete turn:

1. Choose a repeatable starting point on your mouse pad.
2. Measure the physical distance required for a 360-degree turn in your game.
3. In Static Flick, move the same distance and adjust `Z` / `X` until it produces one full turn.
4. Repeat once in each direction to catch acceleration or compositor inconsistencies.

Disable desktop mouse acceleration for the closest transfer. On GNOME, the Flat acceleration profile is generally the appropriate starting point for competitive aiming; make that host setting yourself because this project never changes desktop configuration.

## Movement and jumping

`WASD` movement stays on the ground plane. Pressing `Space` applies one upward
impulse; gravity then produces a fixed arc and returns the camera to its normal
eye height. Holding `Space` does not create free flight or a mid-air second
jump, and vertical velocity does not reduce horizontal strafe speed.

## Match field of view

Use `C` / `V` to set the trainer's horizontal field of view. Match the horizontal FOV used by the game at your actual aspect ratio rather than copying a vertical-FOV value. The renderer converts horizontal FOV to the correct vertical projection every frame.

## Projectile practice

Projectile speed is expressed in trainer world units per second, not a game's internal units. Calibrate by feel:

1. Start Projectile Lead with the guide enabled.
2. Adjust `J` / `K` until the visual travel time resembles the hero or weapon you are practicing.
3. Disable the guide with `L` after the lead distance becomes predictable.
4. Use smaller targets with `G` for precision or larger targets with `H` for warmups.

The lead guide solves a constant-velocity intercept exactly. It deliberately excludes acceleration, gravity, drag, and network latency so the lesson remains readable.

## Suggested routine

| Duration | Scenario | Focus |
| ---: | --- | --- |
| 3 minutes | Smooth Track | Relaxed grip and continuous correction |
| 3 minutes | Static Flick | Stop cleanly before firing |
| 3 minutes | Target Switch | Efficient path between targets |
| 3 minutes | Projectile Lead | Predict movement instead of chasing |

Treat scores as a consistency signal, not a reason to tense up. Compare rolling averages in `sessions.csv` rather than a single peak run.
