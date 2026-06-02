<!-- markdownlint-disable MD041 MD028 MD040 MD033-->

> [!NOTE]
> This tool was **coded by hand!**

> [!WARNING]
> This tool is built for personal use. I cannot be held responsible for any damage to your printer. **Use at your own risk.**

> [!CAUTION]
> You can **damage your 3D printer** if not used correctly!

---

<p align="center">
<sub>Made with ❤️ for the 3D printing community</sub>
</p>

# 🖨️ zoffset-adjuster

> **Perfect your first layer. Every single print.**

`zoffset-adjuster` surgically injects Z offset corrections directly into your gcode — no slicer plugins, no firmware changes, no babystep fiddling mid-print.

It adjusts the `z_offset` for early layers only, then automatically reverts it — so your first layer gets the squish it deserves, and the rest of the print stays perfect.

---

## ✨ What it does

1. ▶️ Inserts `SET_GCODE_OFFSET Z_ADJUST={VALUE} MOVE=1` right before your **first layer starts**
   - Negative values → ⬇️ lower nozzle → **more squish**
   - Positive values → ⬆️ raise nozzle → **less squish**

2. ▶️ Inserts `SET_GCODE_OFFSET Z_ADJUST={OPPOSITE_VALUE} MOVE=1` at your chosen layer to **undo the adjustment**

3. 📻 Informs you at what line numbers the corrections were inserted so that you can verify.

Your original `.gcode` file is **never modified** — a new file is always generated. ✅

<img alt="Z offset adjustment inserted into gcode" src=".github/images/gcode-compare1.webp" width="835">
<img alt="Z offset reversion inserted into gcode" src=".github/images/gcode-compare2.webp" width="835">

---

## 🚀 Usage

```bash
# Interactive mode — prompts you for everything
./zoffset-adjuster

# Pass a file, get prompted for the rest
./zoffset-adjuster ./Cube.gcode

# Using the --input flag
./zoffset-adjuster --input ./Cube.gcode

# Fully silent — no prompts, all args required
./zoffset-adjuster --silent --input ./Cube.gcode \
  --first-layer-height 0.26 \
  --layer-height 0.12 \
  --revert-z-offset-at-layer 2 \
  --z-offset -0.015

# Show help
./zoffset-adjuster --help
```

---

## 🎬 Example

Sliced a cube with `first layer height: 0.22mm`, `layer height: 0.12mm`. Want extra squish on the first layer only, reverting at layer 2:

```
➜ ./zoffset-adjuster ./Cube.gcode

> Selected file: ./Cube.gcode
> How much to adjust z_offset by? -0.015 mm
> What is the height of the first layer? 0.220 mm
> What is the height of the other layers? 0.120 mm
> At the start of what layer do you want to undo the Z offset adjustment? 2

Inserting z_offset adjustment at line 108
Inserting z_offset reversion at line 384

./Cube-054539.gcode generated!
Goodbye! 😀
```

---

## ⚙️ Default Settings

A `settings.toml` is generated in the current directory if none is found. Edit it to match your usual print profile so you don't have to type the same values every time:

```toml
z_offset = -0.015
first_layer_height = 0.26
layer_height = 0.2
revert_z_offset_at_layer = 2
```

---

## 📋 Notes

| | |
| --- | --- |
| 🟢 | Klipper only |
| 🟢 | Tested with Orca Slicer 2.3.2 |
| 🟢 | Original `.gcode` is never modified |
| 🟢 | Sanity checks catch bad inputs |
| 🔴 | Does **not** work with adaptive layers |
| 🟡 | Only tested on Linux (for now) |

---

## 📝 Todo

- [ ] Add ability to run via Orca Slicer Post-Processing Script section

---

## 🛠️ Developer Notes

The tool looks for this sequence in the gcode to determine layer boundaries:

```rust
impl GCode {
    const LAYER_CHANGE: &'static str = ";LAYER_CHANGE";
    const CURRENT_PRINT_HEIGHT: &'static str = ";Z:";
    const CURRENT_LAYER_HEIGHT: &'static str = ";HEIGHT:";
}
```

Example sequence from Orca Slicer 2.3.2 (`first layer: 0.22mm`, `layer height: 0.12mm`):

```gcode
;LAYER_CHANGE   ← first layer
;Z:0.22
;HEIGHT:0.22

;LAYER_CHANGE   ← third layer
;Z:0.46
;HEIGHT:0.12
```

---

## 📦 Installation

Grab the latest binary for your platform from the [Releases](../../releases/latest) page:

| Platform | File |
| --- | --- |
| 🐧 Linux (x86_64) | `zoffset-adjuster-x86_64-unknown-linux-gnu` |
| 🪟 Windows (x86_64) | `zoffset-adjuster-x86_64-pc-windows-msvc.exe` |
| 🍎 macOS (Intel) | `zoffset-adjuster-x86_64-apple-darwin` |
| 🍎 macOS (Apple Silicon) | `zoffset-adjuster-aarch64-apple-darwin` |
| <img alt="Raspberry pi logo" src=".github/images/RPI.L.png" width="16" style="vertical-align:middle"> Raspberry Pi 4 / Orange Pi | `zoffset-adjuster-aarch64-unknown-linux-gnu` |

---

## 🪚 Build from Source

```bash
git clone https://github.com/bassamanator/zoffset-adjuster.git
cd zoffset-adjuster
cargo build --release
# find binary under ./target/release/ 
```
