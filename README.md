# Vocab Master - 英语词汇大师 v1.0

## 📦 项目结构

```
vocab-master-final/
├── tauri.conf.json       # Tauri 配置
├── frontend/            # Vue 3 前端
│   ├── public/
│   │   └── index.html
│   ├── src/
│   │   ├── main.js
│   │   ├── App.vue
│   │   ├── router.js
│   │   ├── pages/
│   │   │   ├── DashboardPage.vue
│   │   │   ├── WordsPage.vue
│   │   │   ├── ReviewPage.vue
│   │   │   └── QuizPage.vue
│   │   └── components/
│   │       ├── WordCard.vue
│   │       └── Settings.vue
│   ├── package.json
│   └── vite.config.js
├── src-tauri/          # Rust 后端
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       ├── lib.rs      # Tauri 命令实现
│       └── main.rs
├── data/
│   └── words/         # 词库文件（KyleBing 版）
│       ├── junior.json  # 初中 3223 词
│       ├── high.json    # 高中 6008 词
│       ├── cet4.json   # 四级 7508 词
│       └── cet6.json   # 六级 5651 词
└── icons/             # 应用图标
    ├── 32x32.png
    ├── 128x128.png
    └── 256x256.png
```

## 🚀 本地编译打包

### 环境要求
- Rust (https://rustup.rs)
- Node.js 18+ (https://nodejs.org)
- Tauri CLI: `cargo install tauri-cli --version "^2"`

### 步骤

```bash
# 1. 进入项目目录
cd vocab-master-final

# 2. 安装前端依赖
cd frontend
npm install
cd ..

# 3. 安装根级依赖（Tauri CLI）
npm install

# 4. 开发模式（热重载）
npm run tauri:dev

# 5. 打包为 exe（Windows）
npm run tauri:build
# 输出：src-tauri/target/release/bundle/nsis/vocab-master-setup.exe
```

### 打包后大小估算
- Rust 二进制：~8-12 MB
- 词库（4个学段）：~15 MB
- ECharts CDN：不占包体积
- Tailwind CDN：不占包体积
- 预估总包：**~25-30 MB**

## 🌐 GitHub Actions 自动构建

无需本地安装任何开发工具，推送代码到 GitHub 后 Actions 会自动：

1. 在 Windows 虚拟机上安装 Rust + Node.js
2. 安装前端依赖并构建
3. 生成 NSIS 安装包
4. 上传构建产物为 artifact
5. **清理所有安装的环境**（cargo registry、git cache、target 目录）

### 使用方式

将代码推送到 GitHub 仓库的 `main` 或 `master` 分支，GitHub Actions 会自动触发构建。

构建完成后，在 Actions → 对应 workflow run → Artifacts 中下载 `VocabMaster-Setup`。

## 📂 词库来源

### KyleBing/english-vocabulary
- 初中：3223 词（json/1-初中-顺序.json）
- 高中：6008 词（json/2-高中-顺序.json）
- 四级：7508 词（json/3-CET4-顺序.json）
- 六级：5651 词（json/4-CET6-顺序.json）

### lilinji/English（备选）
- 位于 `data/words/` 下的 `_lilinji.json` 文件
- 如需使用，将 `junior_lilinji.json` 重命名为 `junior.json` 等

## ⚙️ 应用设置

首次运行后，点击左下角 ⚙️ 设置：

1. **模型配置**：填写 API 地址、密钥、模型名称
2. **联网搜索**：可选开启（Bing/DuckDuckGo 双路）
3. **音频缓存**：默认 5 小时自动清理
4. **预置提示词**：可自定义 AI 出题模板

## ✨ 功能清单

| 功能 | 说明 |
|---|---|
| 📚 词库学习 | 初中/高中/四级/六级，精美词卡 |
| 🔊 发音播放 | Free Dictionary API，MP3 缓存 5 小时 |
| ✨ AI 出题 | 调用自定义模型 API 生成选择题 |
| 📖 完形填空 | AI 基于已学单词生成短文 |
| 📊 学习仪表盘 | ECharts 环形图/柱状图/折线图 |
| 🔍 联网搜索 | 自建爬虫，Bing + DuckDuckGo 降级 |
| ⚙️ 全 GUI 配置 | 模型/搜索/提示词均界面配置 |

## 📌 注意事项

- 应用首次运行会自动创建数据目录：
  - Windows: `%APPDATA%\vocab-master\`
  - macOS: `~/Library/Application Support/vocab-master/`
  - Linux: `~/.local/share/vocab-master/`
- 发音缓存目录：`%CACHE%\vocab-master-audio\`
- 配置文件：`%CONFIG%\vocab-master\config.toml`
- 数据库：SQLite（`vocab.db`），存储学习进度和复习记录
