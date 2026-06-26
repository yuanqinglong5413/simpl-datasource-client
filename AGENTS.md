# AGENTS.md

## Cursor Cloud specific instructions

本仓库为 Rust 实现的数据库客户端 GUI（`simpl-datasource-client`）。当前仓库尚处于初始状态，仅包含 `README.md` 与 `LICENSE`，还没有 `Cargo.toml` 或任何应用代码，因此暂时没有可启动的应用。

### 环境与工具链
- Rust 工具链已预装在镜像中（`rustc`/`cargo`/`rustfmt`/`clippy`，stable 1.83）。`CARGO_HOME=/usr/local/cargo`、`RUSTUP_HOME=/usr/local/rustup`。
- 标准命令：构建 `cargo build`、运行 `cargo run`、测试 `cargo test`、格式检查 `cargo fmt --check`、Lint `cargo clippy -- -D warnings`。
- 更新脚本仅在存在 `Cargo.toml` 时执行 `cargo fetch`；在没有 `Cargo.toml` 之前为空操作。

### GUI / 数据库依赖（非显而易见）
- 已具备：`X11`、`xcb` 系统库，且有虚拟桌面 `DISPLAY=:1`，适合基于 `winit`/`egui`/`eframe` 的软件渲染 GUI 做手动验证。
- 缺失（按所选技术栈再按需安装，勿提前盲装）：
  - Tauri 方向需要 `gtk+-3.0`、`webkit2gtk-4.1`（`libgtk-3-dev`、`libwebkit2gtk-4.1-dev`）。
  - Wayland 后端需要 `wayland-client`、`xkbcommon`（`libwayland-dev`、`libxkbcommon-dev`）。
  - 数据库驱动按需安装：PostgreSQL `libpq-dev`、SQLite `libsqlite3-dev`、TLS `libssl-dev`、`pkg-config`（已安装）。
  - 安装后请同步更新本仓库的更新脚本与本文件。
