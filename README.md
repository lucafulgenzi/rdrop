# rdrop

**rdrop** is a lightweight terminal dropdown utility for **Hyprland**, built on top of `hyprctl`.  
It allows you to toggle a terminal that slides in from any screen edge (top, right, bottom, left), similar to tools like Yakuake or Guake — but fully integrated with Hyprland's dynamic window management.

Everything is fully configurable via a simple YAML file.

---

## ✨ Features

- [X] Edge-based positioning (Top / Right / Bottom / Left)  
- [X] Simple YAML configuration  
- [X] Zero dependencies beyond `hyprctl`
- [ ] Animation when terminal appear

---

## 📦 Installation

### Repositories

[![Packaging status](https://repology.org/badge/vertical-allrepos/rdrop.svg)](https://repology.org/project/rdrop/versions)

### Install with Cargo

```shell
git clone https://github.com/lucafulgenzi/rdrop
cd rdrop
cargo install --path .
```

### Install with yay (AUR)

```shell
yay -S rdrop-bin
```

---

## ⚙️ Usage

Run:

``` shell
rdrop
```

For best usage put into your `hyprland.conf` configs:
```shell
float_switch_override_focus = 0

exec-once = rdrop 

bind = $mainMod SHIFT, Q, exec, rdrop
```

---

## 🛠 Configuration

Default config path:

    ~/.config/rdrop/rdrop.yaml

Pass your custom configuration:

```shell
rdrop --config <config_path>
```

Example configuration:

``` yaml
---
terminal: kitty
class: kitty-drop
width: 80
height: 60
gap: 60
position: T
```

### Field Description

| Field | Type  |Description|
|---|---|---|
|`terminal` |string|Terminal application to run|
|`class` |string|Window class identifier|
|`width` |int|Monitor width percentage|
|`height`|int|Monitor height percentage|
|`gap`|int|Pixel distance from the screen edge|
|`position`|enum|Edge position: `T`, `R`, `B`, `L`|

---

📚 References & Inspirations

This project was inspired by:
- [hrop](https://github.com/Schweber/hdrop)

---

## 📄 License

GPL-3.0 License.

