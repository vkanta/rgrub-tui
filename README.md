# rgrub-tui

A high-performance, terminal-based GRUB configuration utility written in Rust. 🚀

`rgrub-tui` is a clone of the popular `grub-customizer`, reimagined as a lightweight Terminal User Interface (TUI) using [Ratatui](https://ratatui.rs/). It allows you to browse boot entries and modify system GRUB settings directly from your terminal with safety and speed.

## ✨ Features

*   **Menu Management**: Browse through kernel boot entries and nested submenus.
*   **Settings Editor**: View and edit `/etc/default/grub` key-value pairs in a dedicated interface.
*   **Interactive TUI**: Smooth, immediate-mode rendering with full keyboard navigation.
*    fast & Safe: Built with Rust for memory safety and high performance.
*   **Safety First**: Includes confirmation prompts before writing any changes to your system configuration files.

## 🛠️ Tech Stack

*   **Language**: [Rust](https://www.rust-lang.org/)
*   **TUI Framework**: [Ratatui](https://github.com/ratatui-org/ratatui)
*   **Terminal Backend**: [Crossterm](https://github.com/crossterm-rs/crossterm)
*   **Error Handling**: [Anyhow](https://github.com/dtolnay/anyhow)

## 🚀 Getting Started

### Prerequisites

Ensure you have the Rust toolchain installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installation

Clone this repository and build with Cargo:

```bash
git clone https://github.com/vkanta/rgrub-tui.git
cd rgrub-tui
cargo build --release
```

### Running the App

Since modifying GRUB configuration requires elevated privileges, you must run the binary with `sudo`:

```bash
sudo ./target/release/rgrub-tui
```

## 🎮 Controls

| Key | Action |
| :--- | :--- |
| `↑` / `↓` | Navigate through lists |
| `Enter` | Select entry or Apply edit |
| `s` | Toggle between **Menu** and **Settings** views |
| `e` | Enter **Edit Mode** for a setting |
| `Shift + S` | Trigger **Save** request |
| `Esc` / `c` | Cancel/Go back |
| `q` | Quit application |

## ⚠️ Disclaimer

**Use this tool with caution.** Modifying your GRUB configuration can prevent your system from booting if incorrect values are provided. Always ensure you have a backup of your configuration or a live recovery USB available.

---
*Built by [vkanta](https://github.com/vkanta)*
