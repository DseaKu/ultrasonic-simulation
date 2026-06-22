# Bevy Ultrasonic Simulation

A 2D analytical ultrasonic sensor simulation environment built using the
**Bevy** game engine (v0.19) and the **Avian2D** physics engine (v0.7).

This project provides a physical and mathematical model for simulating real-time
ultrasonic sensor dynamics. It covers raycasting-based spatial query collection,
relative velocity derivation, Doppler shift estimation, Gaussian pulse signal
synthesis, and constructive/destructive acoustic wave interference.

> [!WARNING] **Work in Progress:** This project is currently under active
> development. Not all features are fully implemented yet.

---

## 🚀 Features & Architecture

The simulator is designed to run in a 2D environment, leveraging physics engines
strictly for spatial queries. The simulation follows a modular component-driven
design:

- **`UltrasonicSensor` Component:** Represents the transducer, managing base
  frequency ($f_t$), speed of sound ($c$), pulse width ($\sigma$), and ray
  configuration parameters (beam angle, ray count).
- **`Reflector` Component:** Identifies targets and objects in the environment
  that reflect acoustic waves.
- **Cone Raycasting:** Simulates physical ultrasonic beam spread via a bundle of
  2D physics-based raycasts.
- **Analytical Signal Processing:** Synthesizes the physical carrier waveform
  with Doppler-shifted frequencies and Gaussian envelopes, modeling constructive
  and destructive acoustic interference.

## 🛠️ Getting Started

### Prerequisites

Ensure you have [Rust](https://rustup.rs/) installed.

### Running the Project

To compile and launch the simulation:

```bash
cargo run
```

---

## 📖 Mathematical Foundation & Blueprint

The full mathematical logic and implementation details can be found in the
[Bevy Ultrasonic Raycasting Implementation Guide](doc/guide.md). Below is a
summary of the core physics pipeline:

### 1. Spatial Data Collection

In the sensor update loop, a bundle of physics rays is emitted from the sensor's
position across the configured beam angle. For each hit, the simulator
registers:

- Hit entity
- Hit point coordinates
- Distance to target ($d_{\text{current}}$)

### 2. Physical Calculations per Hit

- **Time of Flight ($t_{\text{delay}}$):**
  $$t_{\text{delay}} = \frac{2d_{\text{current}}}{c}$$
- **Relative Velocity ($v$):**
  $$v = \frac{d_{\text{current}} - d_{\text{previous}}}{\Delta t}$$
- **Doppler Shift ($f_r$):** $$f_r = f_t \left( \frac{c - v}{c + v} \right)$$

### 3. Signal Synthesis & Superposition

Rather than simple envelope summation, the simulator calculates high-frequency
wave interactions to capture interference patterns:

- **Gaussian Envelope ($E_i(t)$):**
  $$E_i(t) = e^{-\frac{(t - t_{\text{delay},i})^2}{2\sigma^2}}$$
- **Individual Waveform ($S_i(t)$):**
  $$S_i(t) = E_i(t) \cdot \cos(2\pi f_{r,i} (t - t_{\text{delay},i}))$$
- **Superposition ($S_{\text{total}}(t)$):**
  $$S_{\text{total}}(t) = \sum_{i=1}^{N} S_i(t)$$
