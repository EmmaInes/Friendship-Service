const SUPPORTED_LOCALES = ['en', 'es', 'pt'];
const DEFAULT_LOCALE = 'en';
const STORAGE_KEY = 'fs_locale';

let currentLocale = DEFAULT_LOCALE;
let translations = {};

async function loadLocale(locale) {
  if (translations[locale]) return;
  const res = await fetch(`/i18n/locales/${locale}.json`);
  translations[locale] = await res.json();
}

export async function initI18n() {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved && SUPPORTED_LOCALES.includes(saved)) {
    currentLocale = saved;
  }
  await Promise.all([
    loadLocale('en'),
    currentLocale !== 'en' ? loadLocale(currentLocale) : Promise.resolve(),
  ]);
  document.documentElement.lang = currentLocale;
}

export function t(key, params = {}) {
  const str =
    translations[currentLocale]?.[key] ||
    translations['en']?.[key] ||
    key;
  return str.replace(/\{(\w+)\}/g, (_, name) => params[name] ?? '');
}

export function getLocale() {
  return currentLocale;
}

export async function setLocale(locale) {
  if (!SUPPORTED_LOCALES.includes(locale)) return;
  currentLocale = locale;
  localStorage.setItem(STORAGE_KEY, locale);
  await loadLocale(locale);
  document.documentElement.lang = locale;
  window.dispatchEvent(new HashChangeEvent('hashchange'));
}

export function getSupportedLocales() {
  return SUPPORTED_LOCALES;
}

const ERROR_MAP = {
  'Invalid credentials': 'error.invalidCredentials',
  'Email or username already taken': 'error.emailOrUsernameTaken',
  'Not authenticated': 'error.notAuthenticated',
  'Invalid or expired token': 'error.invalidToken',
  'No account found with that email and username combination': 'error.noAccountFound',
  'Cannot request your own service': 'error.ownService',
  'Service not found': 'error.serviceNotFound',
  'Only the requester can cancel': 'error.onlyRequesterCancel',
  'Only the provider can update this status': 'error.onlyProviderUpdate',
  'Google token verification failed': 'error.googleTokenFailed',
  'This account uses Google Sign-In': 'error.googleAccountOnly',
};

export function translateError(backendError, fallbackKey) {
  const key = ERROR_MAP[backendError];
  return key ? t(key) : (fallbackKey ? t(fallbackKey) : backendError);
}
