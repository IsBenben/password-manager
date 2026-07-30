# Password Manager

A cross-platform desktop password manager built with Tauri (Rust + Vue 3), featuring local encrypted storage, Git cloud backup, and browser auto-fill extension.

## Features

- **AES-256-GCM Encryption** — All sensitive fields (password, email, phone, 2FA seed) are individually encrypted with PBKDF2-derived keys + AES-256-GCM, each with a random IV
- **Master Password Protection** — Viewing sensitive data requires the master password; session timeout is configurable (default 30 min)
- **Password Strength Validation** — Master password must be ≥12 characters with uppercase, lowercase, digit, and symbol
- **Search & Sort** — Fuzzy search by site URL, username, or note; sort by name, date, or site (favorites first)
- **Masked Display** — Sensitive fields are hidden by default; reveal requires authentication
- **Quick Copy** — One-click copy username/password to clipboard, auto-cleared after 30 seconds
- **TOTP Code Generator** — Built-in TOTP generator (RFC 6238) with countdown bar and next-code preview
- **Git Cloud Backup** — One-click Push/Pull sync to a private Git repository
- **Import/Export** — Supports JSON and CSV formats, compatible with Bitwarden CSV
- **Auto-backup Before Import** — Automatically backs up current data to `.json.bak` before JSON import
- **Browser Extension** — Detects password fields and fills username/password/TOTP with one click
- **Extension Right-click Menu** — Right-click any input → "Fill Username" / "Fill Password"
- **Extension Shortcut** — Ctrl+Shift+L to auto-fill credentials for the current site
- **Local HTTP API** — Extension communicates with the desktop app via `127.0.0.1:33445`
- **Password Generator** — Random passwords (custom length/character sets) and readable passphrases (2-10 words, capitalize/append number/custom separator)
- **Keyboard Shortcuts** — Ctrl+F search, Ctrl+N new entry, Esc close modal
- **Collapsible Sidebar** — Left navigation collapses to a narrow strip, freeing reading space
- **Skeleton Loading** — Animated skeleton placeholders during list/detail loading, not plain text
- **Custom Font** — Switch between system fonts or set a custom font family
- **Multi-language** — English / Chinese real-time switching
- **Multi-email Support** — Each entry supports multiple emails with a primary designation
- **Autofill Mode** — Choose what to fill as the username: the stored username, primary email, phone number, or nothing
- **Dark Mode** — Light/dark/system-follow themes with customizable accent color
- **Audit Logging** — Sensitive operations (add/edit/delete/password change/import) are logged

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Frontend | Vue 3 + TypeScript + Vite |
| Desktop Framework | Tauri 2.x (Rust) |
| Encryption | PBKDF2 (HMAC-SHA256, 600k iterations) + AES-256-GCM |
| Data Storage | Single JSON file (`~/.password-manager/data.json`) |
| Version Control | git2-rs |
| HTTP Service | axum (local 127.0.0.1:33445) |
| Browser Extension | Manifest V3, TypeScript |
| TOTP | totp-rs |

## Data File Structure

Data file location: `~/.password-manager/data.json`

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
      "note": "plaintext note",
      "autofill_mode": "default",
      "category": "Work",
      "favorite": false,
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

## Custom Configuration

The following settings can be adjusted via the Settings UI:

| Setting | Default | Description |
|---------|---------|-------------|
| `git_remote` | `""` | Git remote URL for Push/Pull sync |
| `font_family` | `"system-ui, -apple-system, ..."` | UI font family |
| `session_timeout_minutes` | `30` | Session timeout in minutes (range 1–1440) |

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) ≥ 18
- [Rust](https://www.rust-lang.org/) ≥ 1.77
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Windows, includes C++ toolchain)
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (built-in on Windows 10 1803+)

### Install & Run

```bash
# Clone the repository
git clone <your-repo-url>
cd password-manager

# Install frontend dependencies
npm install

# Development mode
npx tauri dev

# Production build
npx tauri build
```

Build artifacts are in `src-tauri/target/release/`:
- `app.exe` — Portable executable
- `password-manager_0.1.0_x64.msi` — Installer (requires WiX)

### Browser Extension

The extension code is in the `extension/` directory. To use it:

1. Open Chrome → `chrome://extensions`
2. Enable "Developer mode"
3. Click "Load unpacked" → select the `extension/` directory
4. Ensure the desktop app is running (HTTP service listens on `127.0.0.1:33445`)

### Custom CSS Selectors

Some websites may have login forms that the extension cannot auto-detect. You can set custom CSS selectors manually:

1. In the extension popup, click the gear icon ⚙ to open settings
2. Fill in custom CSS selectors for the username and password fields (e.g. `input[name="un"]`, `#login-username`)
3. Click "Save" — selectors are applied to the current tab immediately

Settings are persisted in `chrome.storage.local` and auto-applied each time the popup opens. Leave fields empty to use the default auto-detection logic.

Common examples:

| Website | Username Selector | Password Selector |
|---------|------------------|------------------|
| MC百科 (Minecraft Wiki) | `input[name="un"]` | `input[name="pw"]` |
| Generic name match | `input[name="username"]` | `input[name="password"]` |
| Generic ID match | `#login-username` | `#login-password` |

## Project Structure

```
password-manager/
├── src/                          # Vue 3 Frontend
│   ├── main.ts                   # Entry point
│   ├── App.vue                   # Root component
│   ├── router/index.ts           # Route configuration
│   ├── stores/                   # Pinia + reactive modules
│   │   ├── authStore.ts          # Authentication & session
│   │   ├── passwordStore.ts      # Password CRUD
│   │   ├── configStore.ts        # Configuration
│   │   ├── themeStore.ts         # Dark mode & accent color
│   │   ├── i18nStore.ts          # Internationalization
│   │   ├── dialogStore.ts        # Confirm/prompt dialog
│   │   └── toastStore.ts         # Toast notifications
│   ├── views/
│   │   ├── LoginView.vue         # Login / initialization
│   │   ├── PasswordListView.vue  # Password list (search, sort, shortcuts)
│   │   ├── PasswordDetailView.vue# Detail view (copy, TOTP)
│   │   └── SettingsView.vue      # Settings page
│   └── components/
│       ├── AppSidebar.vue        # Collapsible sidebar
│       ├── ConfirmDialog.vue     # Reusable confirm/prompt dialog
│       ├── Toast.vue             # Toast notification container
│       ├── PasswordForm.vue      # Add / edit form
│       ├── PasswordGenerator.vue # Password/passphrase generator
│       └── PasswordStrengthMeter.vue # Password strength indicator
│
├── src-tauri/                    # Rust Backend
│   └── src/
│       ├── main.rs               # Windows entry point
│       ├── lib.rs                # Tauri app bootstrap
│       ├── models.rs             # Data models
│       ├── crypto.rs             # Encryption module
│       ├── storage.rs            # Storage layer
│       ├── commands.rs           # Tauri commands
│       ├── wordlist.rs           # Passphrase word list (~1500 words)
│       ├── git_sync.rs           # Git sync
│       └── http_service.rs       # HTTP service
│
├── extension/                    # Chrome Extension
│   ├── manifest.json             # Manifest V3
│   ├── background.js             # Service Worker (context menus, message routing)
│   ├── content.js                # Content script (input tracking, auto-fill)
│   ├── popup.html                # Popup window
│   └── popup.js                  # Popup logic
│
├── scripts/                      # Utility scripts
│   └── check-i18n.mjs            # i18n coverage checker
│
└── package.json
```

## API Reference

Tauri IPC commands (`#[tauri::command]`):

| Command | Parameters | Description |
|---------|-----------|-------------|
| `list_entries` | `search?, category?, favorite?` | Fuzzy search by site/note, filter by category/favorite |
| `list_categories` | - | List all categories with counts |
| `toggle_favorite` | `id` | Toggle favorite status |
| `get_entry` | `id, password` | Get single entry with decrypted fields |
| `add_entry` | `entry, password` | Add a new entry (validates password and required fields) |
| `edit_entry` | `id, entry, password` | Edit an existing entry |
| `delete_entry` | `id` | Delete an entry |
| `change_master_password` | `old, new` | Change master password and re-encrypt all data |
| `init_password` | `password` | Initialize salt and encryption system |
| `verify_password` | `password` | Verify master password correctness |
| `get_config` | - | Get configuration (Git/font/timeout) |
| `update_config` | `config` | Update configuration |
| `git_push` | `message?` | Commit and push to Git |
| `git_pull` | - | Pull from Git |
| `generate_totp` | `secret, step_offset?` | Generate TOTP code |
| `generate_password` | `length, use_upper, use_lower, use_digits, use_symbols, exclude_confusing` | Generate random password |
| `generate_passphrase` | `word_count?, separator?, capitalize?, append_number?` | Generate readable passphrase |
| `export_json` | `path` | Export encrypted data to a file path |
| `export_csv` | `path, password` | Export as CSV (Bitwarden-compatible) |
| `import_json` | `path, password` | Import from JSON file (auto-backup before overwrite) |
| `import_csv` | `path, password` | Import from CSV (auto-detects Bitwarden format) |

Extension HTTP API:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Check if desktop app is running |
| `/api/decrypt` | POST | Decrypt credentials for the current site |

## Security Design

1. **Key Derivation**: PBKDF2-HMAC-SHA256, 600,000 iterations, 16-byte random salt
2. **Field Encryption**: AES-256-GCM, each sensitive field uses a unique 12-byte random nonce
3. **Storage Format**: `salt(16B) + nonce(12B) + ciphertext + tag(16B)`, all Base64-encoded
4. **Session Management**: Frontend clears decrypted data after use; Rust uses `zeroize` for memory sanitization; session timeout is configurable
5. **Brute Force Protection**: Delays on failed verification, salt rotation on password change
6. **Transport Security**: HTTP service binds to `127.0.0.1` only, not exposed externally
7. **Audit Logging**: Sensitive operations (add/edit/delete/password change/import) are logged

## Development Roadmap

- [x] Phase 1: Core Backend (Rust) — Data model, encryption, CRUD, Git sync
- [x] Phase 2: Desktop Frontend (Vue) — List, search, detail, settings
- [x] Phase 3: HTTP Service & Extension — Local API, browser auto-fill
- [x] Phase 4: Session config, import/export, dark mode, CSV, Toast, custom dialogs
- [x] Phase 5: Right-click menu, shortcuts, sorting, skeleton screens, audit log, clipboard mgmt, passphrases
- [ ] Phase 6: Integration testing & optimization — E2E tests, performance, security audit

## License

MIT
