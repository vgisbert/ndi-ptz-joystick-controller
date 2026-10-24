---
name: Tactical Broadcast Console
colors:
  surface: '#111319'
  surface-dim: '#111319'
  surface-bright: '#36393f'
  surface-container-lowest: '#0b0e13'
  surface-container-low: '#191c21'
  surface-container: '#1d2025'
  surface-container-high: '#272a30'
  surface-container-highest: '#32353a'
  on-surface: '#e1e2e9'
  on-surface-variant: '#b9cacb'
  inverse-surface: '#e1e2e9'
  inverse-on-surface: '#2e3036'
  outline: '#849495'
  outline-variant: '#3b494b'
  surface-tint: '#00dbe9'
  primary: '#dbfcff'
  on-primary: '#00363a'
  primary-container: '#00f0ff'
  on-primary-container: '#006970'
  inverse-primary: '#006970'
  secondary: '#ffd799'
  on-secondary: '#432c00'
  secondary-container: '#feb300'
  on-secondary-container: '#6a4800'
  tertiary: '#fff3f2'
  on-tertiary: '#680014'
  tertiary-container: '#ffcdcc'
  on-tertiary-container: '#bf002e'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#7df4ff'
  primary-fixed-dim: '#00dbe9'
  on-primary-fixed: '#002022'
  on-primary-fixed-variant: '#004f54'
  secondary-fixed: '#ffdeac'
  secondary-fixed-dim: '#ffba38'
  on-secondary-fixed: '#281900'
  on-secondary-fixed-variant: '#604100'
  tertiary-fixed: '#ffdad9'
  tertiary-fixed-dim: '#ffb3b3'
  on-tertiary-fixed: '#400009'
  on-tertiary-fixed-variant: '#920021'
  background: '#111319'
  on-background: '#e1e2e9'
  surface-variant: '#32353a'
typography:
  headline-xl:
    fontFamily: Space Grotesk
    fontSize: 28px
    fontWeight: '700'
    lineHeight: 34px
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Space Grotesk
    fontSize: 20px
    fontWeight: '600'
    lineHeight: 26px
    letterSpacing: -0.01em
  headline-md:
    fontFamily: Space Grotesk
    fontSize: 16px
    fontWeight: '600'
    lineHeight: 22px
    letterSpacing: 0em
  headline-sm:
    fontFamily: Space Grotesk
    fontSize: 14px
    fontWeight: '600'
    lineHeight: 18px
    letterSpacing: 0.02em
  body-lg:
    fontFamily: Hanken Grotesk
    fontSize: 15px
    fontWeight: '400'
    lineHeight: 22px
  body-md:
    fontFamily: Hanken Grotesk
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 18px
  body-sm:
    fontFamily: Hanken Grotesk
    fontSize: 11px
    fontWeight: '400'
    lineHeight: 16px
  label-lg:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '600'
    lineHeight: 16px
    letterSpacing: 0.05em
  label-md:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '500'
    lineHeight: 14px
    letterSpacing: 0.04em
  label-sm:
    fontFamily: JetBrains Mono
    fontSize: 9px
    fontWeight: '500'
    lineHeight: 12px
    letterSpacing: 0.06em
  telemetry-display:
    fontFamily: JetBrains Mono
    fontSize: 18px
    fontWeight: '700'
    lineHeight: 22px
    letterSpacing: 0.08em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  gutter: 0.75rem
  margin: 1rem
  space-xs: 0.25rem
  space-sm: 0.5rem
  space-md: 0.75rem
  space-lg: 1rem
  space-xl: 1.5rem
---

## Brand & Style

This design system models high-reliability physical studio gear—specifically rackmount routing units, production switcher desks, and heavy cast-metal camera control surfaces. The UI operates in dark broadcast control rooms and live production trucks where ambient light is minimal and visual fatigue must be avoided during 12-hour continuous shifts.

The aesthetic fuses industrial tactile skeuomorphism with precision instrumentation telemetry. Visual weight is anchored by dark milled graphite, anodized aluminum bezels, backlit silicone keycaps, and recessed high-contrast OLED panels. Controls evoke physical dampening, spring-back tactile action, and clear illumination states: off, standby amber, active cyan, and live broadcast tally red. 

The interface avoids transient mobile trends or ephemeral minimalism. Every control occupies a dedicated, deterministic coordinate space to support rapid muscle memory, zero-latency feedback, and fail-safe field operations.

## Colors

The palette reproduces the lighting environment of high-end rack equipment and studio consoles:

- **Primary (`#00F0FF` - Precision Cyan):** Active channel selection, operational telemetry reticles, pan/tilt vector highlights, and locked servo states. Carries high luminance against the dark foundation without causing retinal blooming.
- **Secondary (`#FFB300` - Tactical Amber):** Preset recall readiness, warning thresholds, standby camera assignments, manual override focus, and rate-limiting limits.
- **Tertiary (`#FF1744` - Tally Red):** Live program broadcast state, active memory store/overwrite, streaming output lock, and emergency halt.
- **Neutral (`#0E1116` - Anodized Charcoal Base):** The deep metallic chassis baseline. Extended across structured tonal increments:
  - Chassis Base: `#0E1116`
  - Milled Recess / Panel Tray: `#090B0E`
  - Control Surfaces & Modules: `#161B22`
  - Bezel & Division Rails: `#242C38`
  - Machined Highlights & Edge Gleams: `#3A4454`
- **Network / Protocol Accent (`#2979FF` - NDI Cyber Blue):** Dedicated protocol indicators, stream health, packet synchronization status, and discovery trees.
- **Telemetry Emissive Screens (`#04070A` Screen Ground with `#39FF14` or `#00F0FF` segments):** Replicates OLED micro-displays set into machined hardware cutouts.

## Typography

The type system prioritizes instantaneous parsing under critical timing constraints.

- **Primary Display & Headings (`Space Grotesk`):** Provides sharp, mechanical authority for console module headers, bank titles, and camera selector arrays.
- **Interface & Operational Copy (`Hanken Grotesk`):** Clean, modern humanist sans-serif used for setting descriptions, modal dialogues, input fields, and hardware configuration menus. Remains legible at small point sizes on high-density Windows displays.
- **Telemetry & Control Labels (`JetBrains Mono`):** Dedicated to technical metadata: IP addresses, Pan/Tilt/Zoom coordinates, focal distance, exposure indices (iris/gain/shutter), NDI stream bitrates, and hardware port labeling. All coordinates and numeric readouts use tabular numbers (`tnum`) to eliminate spatial shifting as values rapidly update.
- **Text Transformations:** Hardware labels and module captions are strictly rendered in uppercase with wide tracking (`0.05em` to `0.08em`) to mimic stamped and laser-etched faceplates.

## Layout & Spacing

The layout is built for native multi-monitor Windows broadcast workstations, with native optimization for 16:9 and 21:9 ultra-wide displays (minimum target: 1920x1080; primary target: 2560x1440 and 3840x2160). It uses a fixed, docked modular panel system that forbids fluid auto-collapsing. Panels stay anchored to predetermined physical quadrants:

1. **Top Telemetry Header:** Persistent global status (NDI network health, master clock, sync mode, tally summary).
2. **Left Bay (Camera Bank Selection):** 8-to-24 channel direct-access hardware button matrix.
3. **Center Bay (Live View & Tactical Vector Radar):** Low-latency NDI return feed, pan/tilt reticle overlay, zoom depth scale, tracking vector indicators.
4. **Right Bay (Physical Actuator Emulation):** Digital twin dials for Focus, Iris, White Balance, Pan/Tilt speed, and curve shaping.
5. **Bottom Deck (Preset & Trajectory Matrix):** 16 illuminated macro pads representing spatial coordinates, stop points, and cruise tracks.

Panel spacing relies on high-density layout constraints (`0.75rem` / 12px gutters) with inset wells to conserve screen real estate while maintaining clean separation between critical live functions.

## Elevation & Depth

Visual depth is achieved through mechanical layering and controlled light sources rather than diffuse ambient shadows:

- **Chassis Level 0 (Recessed Chassis Well):** `#090B0E`. Used behind control buttons and display inserts. Deep inset bevel with a 1px inner stroke: `inset 0 2px 4px rgba(0, 0, 0, 0.8), inset 0 0 1px #000000`.
- **Chassis Level 1 (Surface Faceplates):** `#161B22`. The main panel surface with fine brushed metallic texture simulation. Surrounded by a 1px border (`#242C38`) and a crisp drop shadow of `0 4px 12px rgba(0, 0, 0, 0.6)`.
- **Chassis Level 2 (Rotary Dials & Raised Silicone Caps):** `#1F2633`. Tactile items raised above the faceplate. Elevated via top-edge directional highlights (`1px solid #3A4454`) and cast directional drop shadows: `0 3px 6px rgba(0, 0, 0, 0.7)`.
- **Level 3 (Illumination & Active Glows):** Active elements emit light into adjacent surfaces:
  - Cyan Active: `box-shadow: 0 0 12px rgba(0, 240, 255, 0.45), inset 0 0 6px rgba(0, 240, 255, 0.6)`
  - Amber Standby: `box-shadow: 0 0 12px rgba(255, 179, 0, 0.45), inset 0 0 6px rgba(255, 179, 0, 0.6)`
  - Tally Red: `box-shadow: 0 0 16px rgba(255, 23, 68, 0.6), inset 0 0 8px rgba(255, 23, 68, 0.7)`
- **OLED Surface Depth:** Set into a dark recessed pocket with an ultra-thin perimeter glass bezel (`#10151D`) and a glass sheen gradient overlay at 2% opacity.

## Shapes

The geometry uses tight, machined corner profiles (`roundedness: 1`). Radii reflect precision industrial manufacturing:

- **Modules and Rack Enclosures:** Outer corners use `0.5rem` (8px) rounding, representing cast-aluminum hardware casings.
- **Silicone Keycaps & Macro Pads:** `0.25rem` (4px) rounding on raised keys to simulate square-cut rubber buttons with beveled tops.
- **OLED Telemetry Wells:** Crisp `0.25rem` (4px) interior rounding with hard perimeter bezels.
- **Rotary Dials & Vector Gauges:** Pure cylindrical geometries (`rounded-full`) with knurled grip ring borders.
- **LED Indicator Dots:** Pure circular points (6px to 8px diameter) with raised epoxy lens refraction.

## Components

### Silicone Matrix Buttons (Camera Select & Preset Trays)
- **Base State:** Dark carbon silicone (`#1A212B`), crisp 1px perimeter border (`#2A3444`), labeled with uppercase JetBrains Mono in `#8B9BB4`.
- **Depressed/Engaged State:** Translates 1px downward, border shifts to the primary glow color, internal emissive light floods the keycap.
- **Tally Live State:** Key turns glowing `#FF1744` with high-contrast black typography (`#000000`), casting a perimeter tally aura.

### Rotary Aluminum Encoders (Pan/Tilt Speed, Iris, Zoom Trim)
- Rendered as multi-layered circular elements with an outer knurled aluminum friction ring (`#283240`) and an inner dark brushed cap (`#12161F`).
- Features a glowing radial tick mark indicating current rotational trim.
- Encircled by an unbroken 270-degree LED track displaying current level in cyan, amber, or neutral grey.

### OLED Telemetry Screens (IP, Port, Coordinates)
- Inset glass containers (`#04070A`) with hairline borders (`#151C24`).
- Content displays monospaced readout rows:
  - Row 1: `CAM-04 // 192.168.10.144 [NDI-HX]` (Cyan text)
  - Row 2: `P: +142.30°  T: -12.15°  Z: 18.4X` (White high-contrast text)
  - Row 3: `IRIS: F1.8  GAIN: 0dB  SHUTTER: 1/120` (Tactical amber text)
- Subtle horizontal scanline overlay at 3% opacity.

### Toggle Switches & Safety Covers
- 2-state and 3-state industrial toggle switches for motor lock, memory protect, and power busses.
- Safety-critical commands (e.g., Memory Overwrite, Master Reset) feature a diagonal striped hazard shroud (`#FFB300` and `#0E1116`) that must be toggled before triggering.

### PTZ Virtual Coordinate Canvas (Joystick Visualizer)
- Crosshair-oriented directional radar with concentric velocity rings (25%, 50%, 75%, 100%).
- Real-time vector node represented by an illuminated crosshair indicator leaving a subtle phosphor decay trail as the operator deflects the joystick.