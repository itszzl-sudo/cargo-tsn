# Changelog

所有显著项目更改都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added

- **新增 `cargo tsn prepare` 命令**
  - 从 TypeScript 源码自动提取 FFI 函数声明
  - 智能按函数名前缀分组插件（如 `crypto_*`, `http_*`）
  - 生成完整的跨平台文件结构（win/linux/macos）
  - 生成主配置和平台配置文件（TOML 格式）
  - 使用当前时间戳作为插件优先级（确保唯一性）
  - 生成注释形式的 C 函数模板（不生成桩实现）
  - 实时进度条反馈（indicatif）
  - 支持 `--input` 指定输入文件，`--output` 指定输出目录

- **C 文件模板策略**
  - Windows 平台生成注释形式的函数模板
  - Linux/macOS 平台生成占位文件
  - 避免用户忘记实现函数
  - 清晰的 TODO 标记

- **配置格式支持**
  - 支持新跨平台配置格式（`[signatures]`, `[includes]`）
  - 向后兼容旧格式（`[functions]`）
  - 支持优先级字段（`priority`，f64 类型）
  - 支持能力声明（`capabilities`）
  - 支持构建配置（`[build]`）

### Changed

- **禁用 publish 和 install 命令**
  - 原因：缺乏社区贡献
  - 原因：Codeberg 存储方式不合适
  - 原因：无人使用
  - 状态：代码保留但已注释，未来可恢复

- **更新 Extension 结构体**（ts-native 侧）
  - 添加 `includes` 字段（平台配置引用）
  - 添加 `signatures` 字段（函数签名声明）
  - 添加 `capabilities` 字段（能力声明）
  - 添加 `build` 字段（构建配置）
  - 添加 `priority` 字段到 `ExtensionPackage`
  - 所有新字段使用 `#[serde(default)]` 保证兼容性

- **改进开发者体验**
  - C 模板不生成桩函数，避免忘记实现
  - 清晰的注释标记和 TODO 提示
  - 完整的函数签名信息

### Technical Details

- **依赖添加**
  - `regex = "1"` - TypeScript 声明解析
  - `indicatif = "0.17"` - 进度条显示
  - `chrono = "0.4"` - 时间戳生成

- **代码结构**
  - 新增 `src/prepare.rs` 模块（453 行）
  - 实现 TS 解析、FFI 提取、插件分组、文件生成
  - 保留 `generate_function_impl` 以备将来使用

## [0.1.0] - 2026-06-05

### Added

- 初始版本发布
- `cargo tsn new` - 创建 ts-native 项目
- `cargo tsn add` - 添加 crate 依赖
- `cargo tsn func` - 交互式添加 FFI 函数
- `cargo tsn publish` - 发布插件
- `cargo tsn install` - 安装插件
- `cargo tsn list` - 列出本地插件

---

## 版本说明

### Unreleased

**主要特性**：
- `cargo tsn prepare` 命令实现
- 从 TS 源码自动生成 FFI 插件骨架
- 完整的跨平台配置生成
- 时间戳优先级机制

**破坏性更改**：
- 无（向后兼容）

**迁移指南**：
- 无需迁移，新命令为增量添加

### 0.1.0

**初始功能**：
- 项目脚手架
- 依赖管理
- 插件发布/安装

---

[Unreleased]: https://github.com/itszzl-sudo/cargo-tsn/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/itszzl-sudo/cargo-tsn/releases/tag/v0.1.0
