# AGENTS.md

## Cursor Cloud specific instructions

本仓库为 **简源 (SimplSource)** — Rust + Tauri 2 数据库 GUI 客户端。

### 环境与工具链
- Rust stable（见 `rust-toolchain.toml`），标准命令：`cargo build` / `cargo test` / `cargo fmt --check` / `cargo clippy -- -D warnings`。
- 前端：`npm ci` / `npm run build` / `npm run lint`。
- 数据目录可通过环境变量 `SIMPLSOURCE_DATA_DIR` 覆盖。

### 系统依赖（Linux）
- **SSH 隧道**：`libssl-dev`、`pkg-config`
- **Tauri 桌面构建**：`libgtk-3-dev`、`libwebkit2gtk-4.1-dev`、`libayatana-appindicator3-dev`
- **数据库驱动**：PostgreSQL `libpq-dev`、SQLite `libsqlite3-dev`

### 架构速览
- `crates/core` — 连接管理、查询引擎、导入导出、事务
- `crates/ssh` — SSH 本地端口转发
- `crates/driver-*` — PG / MySQL / SQLite 驱动
- `src-tauri` — Tauri IPC
- `src/` — React 前端

### 发布
- 推送 `v*` tag（如 `v0.1.0`）会触发 `.github/workflows/release.yml`，自动构建多平台安装包并创建 GitHub Release。
- 为已有 tag 补发 Release：GitHub Actions → Release → Run workflow，填写 tag 名称。
