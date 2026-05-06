# 餐饮计算器

餐饮行业收款/利润计算工具，支持 Windows + macOS 双平台。

## 功能

- **收款计算** — 输入原价，自动计算收款金额、手续费、支出和净利润
- **多品牌管理** — 每个品牌独立配置收款比例、支出比例、费率、抹零方式
- **阶梯折扣** — 按原价区间配置不同收款比例（如 1-100 元 69%，101-200 元 70%）
- **窗口吸附** — 吸附到"闲鱼管家"窗口边缘，目标移动时自动跟随，位置可配置
- **抹零** — 支持去分（保留到角）、去毛（保留到元）
- **报价复制** — 一键生成报价文案并复制
- **历史记录** — 自动保存计算记录
- **窗口置顶** — 保持窗口在最前面
- **托盘最小化** — 关闭窗口最小化到系统托盘
- **在线更新** — 检测新版本自动下载安装

## 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript + Vite |
| UI | Element Plus |
| 后端 | Rust |
| 平台 API | Win32 (Windows) / CoreGraphics (macOS) |

## 开发

```bash
# 安装依赖
pnpm install

# 启动开发（前端 + Tauri）
pnpm tauri dev

# 仅启动前端
pnpm dev
```

## 打包

```bash
# 生产构建
pnpm tauri build
```

- Windows 产物：`src-tauri/target/release/bundle/nsis/`
- macOS 产物：`src-tauri/target/release/bundle/dmg/`

## 项目结构

```
├── src/                    # 前端代码
│   ├── App.vue             # 主界面（计算器 + 设置 + 吸附）
│   ├── main.ts             # Vue 入口
│   └── utils/
│       └── calculator.ts   # 计算逻辑、品牌管理、阶梯折扣
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── lib.rs          # Tauri 入口、命令注册、托盘
│   │   ├── main.rs         # 程序入口
│   │   └── snap.rs         # 窗口吸附（跨平台实现）
│   ├── capabilities/
│   │   └── default.json    # Tauri 权限配置
│   ├── Cargo.toml          # Rust 依赖
│   └── tauri.conf.json     # Tauri 配置
├── CHANGELOG.md            # 更新日志
└── package.json
```
