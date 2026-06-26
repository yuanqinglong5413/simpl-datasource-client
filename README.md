# 简源 · SimplSource

> 轻量、原生、中文优先的数据库客户端

仓库名：`simpl-datasource-client`

使用 **Rust + Tauri 2 + React** 实现的跨平台数据库 GUI 客户端，参考并对标 DBeaver、Beekeeper Studio、DbGate、TablePlus 等同类开源/商业产品，在启动速度、内存占用与中文体验上形成差异化。

## 产品名

| | |
| --- | --- |
| 中文名 | **简源** |
| 英文名 | **SimplSource** |
| 含义 | 简 — 轻量简洁；源 — 数据源 |

## 技术路线（已确认）

- 桌面框架：Tauri 2
- 前端：React + TypeScript + Vite
- 后端：Rust + Tokio + sqlx
- MVP 数据库：PostgreSQL、MySQL、SQLite

## 规划文档

完整实现规划见 [`docs/plans/2026-06-26-001-feat-rust-db-gui-client-plan.md`](docs/plans/2026-06-26-001-feat-rust-db-gui-client-plan.md)。

## 许可证

MIT
