let customUsernameSelector = '';
let customPasswordSelector = '';
let targetInput: HTMLInputElement | null = null;

document.addEventListener('contextmenu', (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  targetInput = target instanceof HTMLInputElement
    ? target
    : target.closest<HTMLInputElement>('input');
});

function setNativeValue(input: HTMLInputElement, value: string): void {
  const nativeSetter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype, 'value'
  )!.set!;
  nativeSetter.call(input, value);
  input.dispatchEvent(new Event('input', { bubbles: true }));
  input.dispatchEvent(new Event('change', { bubbles: true }));
  input.dispatchEvent(new Event('blur', { bubbles: true }));
}

function findPasswordFields(): boolean {
  if (customPasswordSelector) {
    return document.querySelectorAll(customPasswordSelector).length > 0;
  }
  return document.querySelectorAll('input[type="password"]').length > 0;
}

function findUsernameField(): HTMLInputElement | null {
  if (customUsernameSelector) {
    return document.querySelector<HTMLInputElement>(customUsernameSelector);
  }
  const selectors = [
    'input[autocomplete="username"]',
    'input[name="username"]',
    'input[name="user"]',
    'input[name="un"]',
    'input[name="login"]',
    'input[name="email"]',
    'input[id="login-username"]',
    'input[id*="username"]',
    'input[id*="user-name"]',
    'input[id*="user_name"]',
    'input[id*="login"]',
    'input[placeholder*="user" i]',
    'input[placeholder*="邮箱"]',
    'input[placeholder*="用戶"]',
    'input[placeholder*="用户"]',
  ];
  for (const sel of selectors) {
    const el = document.querySelector<HTMLInputElement>(sel);
    if (el && el.offsetParent !== null) return el;
  }
  const passwordField = document.querySelector<HTMLInputElement>('input[type="password"]');
  if (passwordField) {
    const form = passwordField.closest('form');
    if (form) {
      const textInputs = form.querySelectorAll<HTMLInputElement>('input[type="text"], input[type="email"], input:not([type="hidden"]):not([type="password"]):not([type="submit"]):not([type="button"]):not([type="checkbox"]):not([type="radio"])');
      for (const input of textInputs) {
        if (input !== passwordField && input.offsetParent !== null) return input;
      }
    }
    const parent = passwordField.parentElement;
    if (parent) {
      const siblings = parent.querySelectorAll<HTMLInputElement>('input[type="text"], input[type="email"], input:not([type="hidden"]):not([type="password"]):not([type="submit"]):not([type="button"]):not([type="checkbox"]):not([type="radio"])');
      for (const input of siblings) {
        if (input !== passwordField && input.offsetParent !== null) return input;
      }
    }
  }
  const inputs = document.querySelectorAll<HTMLInputElement>('input[type="text"], input[type="email"], input:not([type="hidden"]):not([type="password"]):not([type="submit"]):not([type="button"]):not([type="checkbox"]):not([type="radio"])');
  for (const input of inputs) {
    if (input.offsetParent !== null) return input;
  }
  return null;
}

function fillCredentials(username: string, password: string, totp?: string): void {
  let passwordFields: NodeListOf<HTMLInputElement>;
  if (customPasswordSelector) {
    passwordFields = document.querySelectorAll<HTMLInputElement>(customPasswordSelector);
  } else {
    passwordFields = document.querySelectorAll<HTMLInputElement>('input[type="password"]');
  }
  if (passwordFields.length === 0) return;

  const usernameField = findUsernameField();
  if (usernameField && username) {
    usernameField.focus();
    setNativeValue(usernameField, username);
  }

  if (passwordFields[0] && password) {
    passwordFields[0].focus();
    setNativeValue(passwordFields[0], password);
  }

  if (totp) {
    const totpSelectors = [
      'input[autocomplete="one-time-code"]',
      'input[name="totp"]',
      'input[id="totp"]',
      'input[name="app_otp"]',
      'input[id="app_totp"]',
      'input[name="otp"]',
      'input[id="otp"]',
      'input[name="two-factor"]',
      'input[name="2fa"]',
      'input[name="verification_code"]',
      'input[id="verification_code"]',
      'input[inputmode="numeric"]:not([type="hidden"])',
      'input[name="otp_token"]',
      'input[id="otp_token"]',
      'input[data-challenge*="totp"]',
      'input[data-challenge*="otp"]',
    ];
    let totpField: HTMLInputElement | null = null;
    for (const sel of totpSelectors) {
      totpField = document.querySelector<HTMLInputElement>(sel);
      if (totpField) break;
    }
    if (totpField) {
      setNativeValue(totpField, totp);
    } else {
      navigator.clipboard.writeText(totp);
    }
  }
}

chrome.runtime.onMessage.addListener((
  message: { type: string; username?: string; password?: string; totp?: string; usernameSelector?: string; passwordSelector?: string },
  _sender: chrome.runtime.MessageSender,
  sendResponse: (response: any) => void
) => {
  if (message.type === 'FILL_CREDENTIALS') {
    fillCredentials(message.username!, message.password!, message.totp);
    sendResponse({ success: true });
  }
  if (message.type === 'CHECK_PASSWORD_FIELDS') {
    sendResponse({ hasPasswordFields: findPasswordFields() });
  }
  if (message.type === 'SET_CUSTOM_SELECTORS') {
    customUsernameSelector = message.usernameSelector || '';
    customPasswordSelector = message.passwordSelector || '';
    sendResponse({ success: true });
  }
  if (message.type === 'FILL_TARGET') {
    const entry = message.entries?.[0];
    if (entry && targetInput) {
      const value = message.fillType === 'password' ? entry.password : entry.username;
      if (value) {
        targetInput.focus();
        setNativeValue(targetInput, value);
      }
    }
    sendResponse({ success: true });
  }
});

if (findPasswordFields()) {
  chrome.runtime.sendMessage({ type: 'PASSWORD_FIELD_DETECTED', url: window.location.href });
}
