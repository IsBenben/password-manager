const POPUP_API_BASE = 'http://127.0.0.1:33445';
const SESSION_DURATION = 5 * 60 * 1000;

const i18nPopup = {
  en: {
    app_title: 'Password Manager',
    desktop_not_running: 'Desktop app not running',
    desktop_connected: 'Desktop connected',
    start_desktop: 'Please start the Password Manager desktop application and ensure the HTTP service is running on port 33445.',
    no_url: 'No valid URL detected',
    enter_password: 'Desktop connected - Enter password to unlock',
    master_password: 'Master Password',
    enter_master_password: 'Enter master password',
    unlock: 'Unlock',
    no_entries: 'No matching entries for this site',
    decryption_failed: 'Decryption failed',
    entries_for: 'Entries for',
    no_username: 'No username',
    click_to_fill: 'Click to fill',
    settings: 'Settings',
    custom_username_selector: 'Custom username selector',
    custom_password_selector: 'Custom password selector',
    custom_selector_placeholder: 'e.g. input[name="un"]',
    save_selectors: 'Save',
    selectors_saved: 'Selectors saved',
    back: 'Back',
    settings_tip: 'Set custom CSS selectors for sites where auto-detection fails. Leave empty to use defaults.',
  },
  zh: {
    app_title: '密码管理器',
    desktop_not_running: '桌面应用未运行',
    desktop_connected: '桌面已连接',
    start_desktop: '请启动密码管理器桌面应用，确保 HTTP 服务正在 33445 端口运行。',
    no_url: '未检测到有效网址',
    enter_password: '桌面已连接 - 输入密码解锁',
    master_password: '主密码',
    enter_master_password: '输入主密码',
    unlock: '解锁',
    no_entries: '未找到匹配的站点条目',
    decryption_failed: '解密失败',
    entries_for: '条目 -',
    no_username: '无用户名',
    click_to_fill: '点击填充',
    settings: '设置',
    custom_username_selector: '自定义用户名选择器',
    custom_password_selector: '自定义密码选择器',
    custom_selector_placeholder: '例如 input[name="un"]',
    save_selectors: '保存',
    selectors_saved: '选择器已保存',
    back: '返回',
    settings_tip: '为自动检测失败的站点设置自定义 CSS 选择器。留空则使用默认值。',
  },
};

function t(key) {
  const lang = navigator.language.startsWith('zh') ? 'zh' : 'en';
  return i18nPopup[lang][key] || i18nPopup.en[key] || key;
}

async function getCurrentTabUrl() {
  const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
  return tabs[0]?.url || '';
}

async function checkHealth() {
  try {
    const resp = await fetch(`${POPUP_API_BASE}/api/health`);
    return resp.ok;
  } catch {
    return false;
  }
}

async function decryptEntries(siteUrl, password) {
  const resp = await fetch(`${POPUP_API_BASE}/api/decrypt`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ site_url: siteUrl, password }),
  });
  if (!resp.ok) throw new Error(t('decryption_failed'));
  return await resp.json();
}

function getSessionExpiry() {
  const stored = sessionStorage.getItem('session_expiry');
  return stored ? parseInt(stored, 10) : 0;
}

function setSessionExpiry(durationMs) {
  sessionStorage.setItem('session_expiry', String(Date.now() + durationMs));
}

function clearSession() {
  sessionStorage.removeItem('session_expiry');
  sessionStorage.removeItem('current_password');
}

function getCachedPassword() {
  return sessionStorage.getItem('current_password') || '';
}

function cachePassword(pwd) {
  sessionStorage.setItem('current_password', pwd);
}

async function loadCustomSelectors() {
  const result = await chrome.storage.local.get(['customUsernameSelector', 'customPasswordSelector']);
  return {
    usernameSelector: result.customUsernameSelector || '',
    passwordSelector: result.customPasswordSelector || '',
  };
}

async function saveCustomSelectors(usernameSelector, passwordSelector) {
  await chrome.storage.local.set({
    customUsernameSelector: usernameSelector,
    customPasswordSelector: passwordSelector,
  });
}

async function applySelectorsToTab(usernameSelector, passwordSelector) {
  const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
  if (tabs[0]?.id) {
    chrome.tabs.sendMessage(tabs[0].id, {
      type: 'SET_CUSTOM_SELECTORS',
      usernameSelector,
      passwordSelector,
    });
  }
}

async function render() {
  const titleEl = document.getElementById('app-title');
  const statusEl = document.getElementById('status');
  const contentEl = document.getElementById('content');
  titleEl.textContent = t('app_title');

  const selectors = await loadCustomSelectors();
  await applySelectorsToTab(selectors.usernameSelector, selectors.passwordSelector);

  const healthy = await checkHealth();
  if (!healthy) {
    statusEl.innerHTML = `<span class="disconnected">${t('desktop_not_running')}</span>`;
    contentEl.innerHTML = `<div class="empty">${t('start_desktop')}</div>`;
    return;
  }
  statusEl.innerHTML = `<span class="connected">${t('desktop_connected')}</span>`;

  const url = await getCurrentTabUrl();
  const domain = extractDomain(url);

  if (!domain) {
    contentEl.innerHTML = `<div class="empty">${t('no_url')}</div>`;
    return;
  }

  const sessionExpiry = getSessionExpiry();
  let password = '';

  if (Date.now() < sessionExpiry) {
    password = getCachedPassword();
  }

  if (!password) {
    statusEl.innerHTML = `<span class="connected">${t('enter_password')}</span>`;
    contentEl.innerHTML = `
      <div class="input-group">
        <label>${t('master_password')}</label>
        <input type="password" id="pwd-input" placeholder="${t('enter_master_password')}" />
      </div>
      <button id="unlock-btn">${t('unlock')}</button>
      <div id="entries"></div>
    `;

    document.getElementById('unlock-btn').addEventListener('click', async () => {
      const pwd = document.getElementById('pwd-input').value;
      if (!pwd) return;

      try {
        const data = await decryptEntries(url, pwd);
        if (data.entries.length === 0) {
          document.getElementById('entries').innerHTML =
            `<div class="empty">${t('no_entries')}</div>`;
          return;
        }
        cachePassword(pwd);
        setSessionExpiry(SESSION_DURATION);
        renderEntries(data.entries, domain);
      } catch (e) {
        document.getElementById('entries').innerHTML =
          `<div class="error">${e.message || t('decryption_failed')}</div>`;
      }
    });
    return;
  }

  try {
    const data = await decryptEntries(url, password);
    if (data.entries.length === 0) {
      contentEl.innerHTML = `<div class="empty">${t('no_entries')}</div>`;
      return;
    }
    renderEntries(data.entries, domain);
  } catch {
    clearSession();
    render();
  }
}

function renderEntries(entries, domain) {
  const contentEl = document.getElementById('content');
  contentEl.innerHTML = `
    <div class="status">${t('entries_for')} ${domain}</div>
    <button id="settings-btn" class="btn-settings" title="${t('settings')}">&#9881;</button>
  `;

  entries.forEach((entry) => {
    const div = document.createElement('div');
    div.className = 'entry';
    div.innerHTML = `
      <div>
        <div class="entry-site">${entry.username || t('no_username')}</div>
        <div class="entry-user">${t('click_to_fill')}</div>
      </div>
    `;
    div.addEventListener('click', () => {
      fillOnPage(entry);
    });
    contentEl.appendChild(div);
  });

  document.getElementById('settings-btn').addEventListener('click', renderSettings);
}

async function renderSettings() {
  const titleEl = document.getElementById('app-title');
  const statusEl = document.getElementById('status');
  const contentEl = document.getElementById('content');

  titleEl.textContent = t('settings');
  statusEl.innerHTML = '';

  const selectors = await loadCustomSelectors();

  contentEl.innerHTML = `
    <p class="settings-tip">${t('settings_tip')}</p>
    <div class="input-group">
      <label>${t('custom_username_selector')}</label>
      <input type="text" id="sel-username" value="${escapeHtml(selectors.usernameSelector)}" placeholder="${t('custom_selector_placeholder')}" />
    </div>
    <div class="input-group">
      <label>${t('custom_password_selector')}</label>
      <input type="text" id="sel-password" value="${escapeHtml(selectors.passwordSelector)}" placeholder="${t('custom_selector_placeholder')}" />
    </div>
    <p id="settings-msg" class="success" style="display:none"></p>
    <button id="save-selectors-btn">${t('save_selectors')}</button>
    <button id="back-btn" class="btn-secondary">${t('back')}</button>
  `;

  document.getElementById('save-selectors-btn').addEventListener('click', async () => {
    const uSel = document.getElementById('sel-username').value.trim();
    const pSel = document.getElementById('sel-password').value.trim();
    await saveCustomSelectors(uSel, pSel);
    await applySelectorsToTab(uSel, pSel);
    const msg = document.getElementById('settings-msg');
    msg.textContent = t('selectors_saved');
    msg.style.display = 'block';
    setTimeout(() => { msg.style.display = 'none'; }, 2000);
  });

  document.getElementById('back-btn').addEventListener('click', render);
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

async function fillOnPage(entry) {
  const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
  if (tabs[0]?.id) {
    chrome.tabs.sendMessage(tabs[0].id, {
      type: 'FILL_CREDENTIALS',
      username: entry.username,
      password: entry.password,
      totp: entry.twofa_secret,
    });
  }
}

function extractDomain(url) {
  try {
    const u = new URL(url);
    return u.hostname;
  } catch {
    return '';
  }
}

document.addEventListener('DOMContentLoaded', render);
