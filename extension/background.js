const API_BASE = 'http://127.0.0.1:33445';

function bgT(key) {
  const msgs = {
    en: { decrypt_failed: 'Failed to decrypt. Is the desktop app running?' },
    zh: { decrypt_failed: '解密失败，桌面应用是否在运行？' },
  };
  const lang = (navigator.language || 'en').startsWith('zh') ? 'zh' : 'en';
  return msgs[lang][key] || msgs.en[key];
}

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: 'pm-root',
    title: '密码管理器',
    contexts: ['editable'],
  });
  chrome.contextMenus.create({
    id: 'pm-fill-user',
    parentId: 'pm-root',
    title: '填充用户名',
    contexts: ['editable'],
  });
  chrome.contextMenus.create({
    id: 'pm-fill-pwd',
    parentId: 'pm-root',
    title: '填充密码',
    contexts: ['editable'],
  });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (!tab?.id || !tab.url) return;
  const { cachedEntries, cachedUrl } = await chrome.storage.session.get(['cachedEntries', 'cachedUrl']);
  if (cachedUrl !== tab.url || !cachedEntries?.length) return;
  const fillType = info.menuItemId === 'pm-fill-pwd' ? 'password' : 'username';
  chrome.tabs.sendMessage(tab.id, { type: 'FILL_TARGET', entries: cachedEntries, fillType });
});

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (sender.id !== chrome.runtime.id) return;
  if (message.type === 'DECRYPT') {
    decryptEntries(message.siteUrl, message.password)
      .then(sendResponse)
      .catch((err) => sendResponse({ error: err.message }));
    return true;
  }
  if (message.type === 'AUTO_FILL') {
    chrome.storage.session.get(['cachedEntries', 'cachedUrl']).then(({ cachedEntries, cachedUrl }) => {
      if (!cachedEntries?.length || !cachedUrl) return;
      chrome.tabs.query({ active: true, currentWindow: true }).then((tabs) => {
        const tab = tabs[0];
        if (tab?.id && tab.url && tab.url === cachedUrl) {
          chrome.tabs.sendMessage(tab.id, { type: 'FILL_TARGET', entries: cachedEntries, fillType: 'username' });
          setTimeout(() => {
            chrome.tabs.sendMessage(tab.id, { type: 'FILL_TARGET', entries: cachedEntries, fillType: 'password' });
          }, 100);
        }
      });
    });
    return false;
  }
});

async function decryptEntries(siteUrl, password) {
  const resp = await fetch(`${API_BASE}/api/decrypt`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ site_url: siteUrl, password }),
  });
  if (!resp.ok) {
    throw new Error(bgT('decrypt_failed'));
  }
  return await resp.json();
}
