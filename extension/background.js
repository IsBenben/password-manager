const API_BASE = 'http://127.0.0.1:33445';

function bgT(key) {
  const msgs = {
    en: { decrypt_failed: 'Failed to decrypt. Is the desktop app running?' },
    zh: { decrypt_failed: '解密失败，桌面应用是否在运行？' },
  };
  const lang = (navigator.language || 'en').startsWith('zh') ? 'zh' : 'en';
  return msgs[lang][key] || msgs.en[key];
}

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (sender.id !== chrome.runtime.id) return;
  if (message.type === 'DECRYPT') {
    decryptEntries(message.siteUrl, message.password)
      .then(sendResponse)
      .catch((err) => sendResponse({ error: err.message }));
    return true;
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
