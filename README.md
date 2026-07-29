# Password Manager

跨平台桌面密码管理器，基于 Tauri（Rust + Vue 3）构建，支持本地加密存储、Git 云端备份、浏览器自动填充插件。

## 功能特性

- **AES-256-GCM 加密** — 所有敏感字段（密码、邮箱、手机号、2FA 种子）使用 PBKDF2 派生密钥 + AES-256-GCM 独立加密，每个字段随机 IV
- **二次密码保护** — 查看敏感信息需输入主密码，会话可自定义自动过期（默认 30 分钟）
- **密码强度校验** — 主密码必须 ≥12 位，包含大小写字母、数字、特殊字符
- **搜索 & 排序** — 按站点/用户名/备注模糊搜索
- **掩码显示** — 敏感字段默认隐藏，点击显示需验证身份
- **TOTP 验证码** — 内建 TOTP 生成器（基于 RFC 6238），含倒计时条和下一组代码预览
- **Git 云端备份** — 一键 Push/Pull 同步到 Git 私有仓库
- **手动导入/导出** — 支持将加密数据导出为 JSON 文件，或从 JSON 文件导入
- **浏览器插件** — 检测密码框，一键填充用户名/密码/TOTP，支持 GitHub / PyPI 2FA
- **HTTP 本地 API** — 插件通过 `127.0.0.1:33445` 与桌面应用通信
- **字体自定义** — 可切换系统字体或自定义字体
- **多语言** — 中文 / English 实时切换
- **密码生成器** — 自定义长度/字符集/排除易混淆字符
- **多邮箱支持** — 每条记录可添加多个邮箱并设置主邮箱

## 技术栈

| 层级 | 技术选型 |
|------|----------|
| 前端桌面 | Vue 3 + TypeScript + Vite |
| 桌面框架 | Tauri 2.x（Rust） |
| 加密 | PBKDF2（HMAC-SHA256, 600k 迭代）+ AES-256-GCM |
| 数据存储 | 单 JSON 文件 (`~/.password-manager/data.json`) |
| 版本控制 | git2-rs |
| HTTP 服务 | axum（本地 127.0.0.1:33445） |
| 浏览器插件 | Manifest V3, TypeScript |
| TOTP | totp-rs |

## 数据文件结构（默认数据）

数据文件路径：`~/.password-manager/data.json`

```json
{
  "version": 1,
  "salt": "<base64 16B salt>",
  "entries": [
    {
      "id": "uuid-v4",
      "site_url": "https://example.com",
      "username": "johndoe",
      "password": "<base64 nonce+ciphertext+tag>",
      "emails_raw": "<base64 nonce+ciphertext+tag>",
      "phone": "<base64 nonce+ciphertext+tag>",
      "twofa_secret": "<base64 nonce+ciphertext+tag>",
      "note": "明文备注",
      "created_at": 1700000000,
      "updated_at": 1700000000
    }
  ],
  "config": {
    "git_remote": "",
    "font_family": "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    "session_timeout_minutes": 30
  }
}
```

## 自定义配置

可通过设置界面调整以下配置项：

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `git_remote` | `""` | Git 远程仓库 URL，用于 Push/Pull 同步 |
| `font_family` | `"system-ui, -apple-system, ..."` | 界面字体 |
| `session_timeout_minutes` | `30` | 会话超时时间（分钟），范围 1–1440 |

## 快速开始

### 前置条件

- [Node.js](https://nodejs.org/) ≥ 18
- [Rust](https://www.rust-lang.org/) ≥ 1.77
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（Windows，含 C++ 工具链）
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)（Windows 10 1803+ 已内置）

### 安装 & 运行

```bash
# 克隆仓库
git clone <your-repo-url>
cd password-manager

# 安装前端依赖
npm install

# 开发模式
npx tauri dev

# 生产构建
npx tauri build
```

构建产物位于 `src-tauri/target/release/`：
- `app.exe` — 可执行文件
- `password-manager_0.1.0_x64.msi` — 安装包（需 WiX）

### 浏览器插件

插件代码位于 `extension/` 目录，使用方式：

1. 打开 Chrome → `chrome://extensions`
2. 开启「开发者模式」
3. 点击「加载已解压的扩展程序」→ 选择 `extension/` 目录
4. 确保桌面应用已运行（HTTP 服务监听 `127.0.0.1:33445`）

### 自定义 CSS 选择器

某些网站的登录表单可能无法被插件自动识别。此时可手动设置自定义 CSS 选择器：

1. 在插件弹窗中，点击齿轮图标 ⚙ 进入设置
2. 填写用户名和密码字段的自定义 CSS 选择器（如 `input[name="un"]`、`#login-username`）
3. 点击「保存」，选择器会自动应用到当前标签页

设置会通过 `chrome.storage.local` 持久化保存，并在每次打开弹窗时自动应用。留空则使用默认的自动检测逻辑。

常见示例：

| 网站 | 用户名选择器 | 密码选择器 |
|------|-------------|-----------|
| MC百科 | `input[name="un"]` | `input[name="pw"]` |
| 通用 name 匹配 | `input[name="username"]` | `input[name="password"]` |
| 通用 ID 匹配 | `#login-username` | `#login-password` |

## 项目架构

```
password-manager/
├── src/                          # Vue 3 前端
│   ├── main.ts                   # 入口文件
│   ├── App.vue                   # 根组件
│   ├── router/index.ts           # 路由配置
│   ├── stores/                   # Pinia 状态管理
│   │   ├── authStore.ts          # 认证 & 会话
│   │   ├── passwordStore.ts      # 密码 CRUD
│   │   └── configStore.ts        # 配置管理
│   ├── views/
│   │   ├── LoginView.vue         # 登录/初始化
│   │   ├── PasswordListView.vue  # 密码列表
│   │   ├── PasswordDetailView.vue# 详情查看
│   │   └── SettingsView.vue      # 设置页
│   └── components/
│       ├── PasswordForm.vue      # 添加/编辑表单
│       └── PasswordGenerator.vue # 密码生成器
│
├── src-tauri/                    # Rust 后端
│   └── src/
│       ├── main.rs               # Windows 入口
│       ├── lib.rs                # Tauri 应用启动
│       ├── models.rs             # 数据模型
│       ├── crypto.rs             # 加密模块
│       ├── storage.rs            # 存储层
│       ├── commands.rs           # Tauri 命令
│       ├── git_sync.rs           # Git 同步
│       └── http_service.rs       # HTTP 服务
│
├── extension/                    # Chrome 扩展
│   ├── manifest.json             # Manifest V3
│   ├── background.js             # 后台 Service Worker
│   ├── content.js                # 内容脚本
│   ├── popup.html                # 弹出窗口
│   └── popup.js                  # 弹出窗口逻辑
│
└── package.json
```

## API 接口

通过 Tauri IPC 调用（`#[tauri::command]`）：

| 命令 | 参数 | 说明 |
|------|------|------|
| `list_entries` | `search?` | 按站点/备注模糊搜索 |
| `get_entry` | `id, password` | 获取单条并解密敏感字段 |
| `add_entry` | `entry, password` | 新增密码记录 |
| `edit_entry` | `id, entry, password` | 编辑密码记录 |
| `delete_entry` | `id` | 删除记录 |
| `change_master_password` | `old, new` | 更改主密码并重加密所有数据 |
| `init_password` | `password` | 初始化盐值和加密系统 |
| `verify_password` | `password` | 验证主密码正确性 |
| `get_config` | - | 获取配置（Git/字体/超时） |
| `update_config` | `config` | 更新配置 |
| `git_push` | `message?` | 提交并推送到 Git |
| `git_pull` | - | 从 Git 拉取 |
| `generate_totp` | `secret, step_offset?` | 生成 TOTP 验证码 |
| `generate_password` | `length, use_upper, use_lower, use_digits, use_symbols, exclude_confusing` | 生成随机密码 |
| `export_json` | `path` | 导出加密数据到指定路径 |
| `import_json` | `path, password` | 从指定路径导入加密数据 |

插件通过本地 HTTP API 通信：

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/health` | GET | 检测桌面应用是否运行 |
| `/api/decrypt` | POST | 解密当前站点的凭据 |

## 安全设计

1. **密钥派生**：PBKDF2-HMAC-SHA256，600,000 次迭代，16 字节随机盐
2. **字段加密**：AES-256-GCM，每个敏感字段独立 12 字节随机 nonce
3. **存储格式**：`salt(16B) + nonce(12B) + ciphertext + tag(16B)`，整体 Base64
4. **会话管理**：前端解密数据使用后及时清空变量，Rust 端使用 `zeroize` 零化内存；会话超时可在设置中自定义
5. **防暴力破解**：验证失败后增加延迟，限制重试频率
6. **传输安全**：HTTP 服务仅监听 `127.0.0.1`，不对外暴露

## 开发计划

- [x] 阶段一：核心后端（Rust）— 数据模型、加密、CRUD、Git 同步
- [x] 阶段二：桌面前端（Vue）— 列表、搜索、详情、设置
- [x] 阶段三：HTTP 服务 & 插件 — 本地 API、浏览器填充
- [x] 阶段四：会话自定义、导入导出、更多特性
- [ ] 阶段五：集成测试与优化 — E2E 测试、性能优化、安全审计

## License

MIT
