# Plan: Internationalization (i18n) — Language Selection

**Languages:** English (default), Spanish, Portuguese  
**Status:** Planned  
**Created:** 2026-03-31

---

## Overview

Add language selection to Friendship&Service so users can switch between English, Spanish, and Portuguese. The approach is zero-dependency, consistent with the project's vanilla JS philosophy.

---

## Architecture

A single `i18n.js` module exports a `t(key)` function. Every page imports `t` and replaces hardcoded strings with `t('key.name')` calls inside template literals. Translation data lives in static JSON files loaded at startup.

When the user switches language, `setLocale()` saves to localStorage and dispatches a `hashchange` event — the app already listens for this, so the entire UI re-renders in the new language with no additional plumbing.

---

## File Structure

```
frontend/src/
  i18n/
    i18n.js              # Core module: t(), initI18n(), setLocale(), translateError()
    locales/
      en.json            # English (~120 keys)
      es.json            # Spanish
      pt.json            # Portuguese
```

### Translation Format

Flat dot-notation keys, organized by page/section:

```json
{
  "nav.browse": "Browse",
  "login.title": "Log In",
  "login.email": "Email",
  "home.welcomeBack": "Welcome back, {name}!",
  "price.free": "Free",
  "error.invalidCredentials": "Invalid credentials"
}
```

Simple `{placeholder}` interpolation: `t('home.welcomeBack', { name: 'Emma' })`

---

## i18n Module (`i18n.js`)

- `initI18n()` — load saved locale from localStorage + English fallback, called once at boot
- `t(key, params)` — look up key in current locale, fall back to English, fall back to raw key
- `setLocale(locale)` — save to localStorage, load locale JSON if needed, dispatch `hashchange` to re-render
- `getLocale()` / `getSupportedLocales()` — state accessors
- `translateError(backendError, fallbackKey)` — map backend English error strings to i18n keys

---

## UI: Language Picker

A `<select>` dropdown in the nav bar, styled to match the nav theme:

```html
<select id="lang-picker" class="lang-picker">
  <option value="en">English</option>
  <option value="es">Español</option>
  <option value="pt">Português</option>
</select>
```

Uses event delegation on the nav (same pattern as the logout button).

---

## Boot Sequence Change

`main.js` becomes async — `initI18n()` must complete before first render:

```js
async function boot() {
  await initI18n();
  renderNav();
  start(app);
}
boot();
```

---

## Page Migration Pattern

Every page file gets two changes:

1. **Import:** `import { t } from '../../i18n/i18n.js';`
2. **Replace strings:** `<h2>Log In</h2>` → `<h2>${t('login.title')}</h2>`

### Shared refactors:
- **`formatPrice()`** — duplicated in 4 files, extract to shared utility using `t('price.free')` etc.
- **`CATEGORIES` array** — duplicated in 3 files, refactor to use translation keys. The `value` sent to the backend remains the English key; only the display label is translated.
- **`STATUS_LABELS`** — use `t('status.pending')` etc.

---

## Backend Error Translation

Backend stays in English. Frontend maps known error strings to i18n keys:

```js
const errorMap = {
  'Invalid credentials': 'error.invalidCredentials',
  'Email or username already taken': 'error.emailOrUsernameTaken',
  // ... ~6 strings total
};
```

No Rust changes needed.

---

## Files Requiring Changes

| File | Changes |
|------|---------|
| `main.js` | Async boot, translate nav, add lang picker |
| `home.js` | ~6 strings |
| `login.js` | ~8 strings |
| `register.js` | ~8 strings |
| `forgot-password.js` | ~10 strings |
| `not-found.js` | ~3 strings |
| `dashboard.js` | ~15 strings + STATUS_LABELS + formatPrice |
| `browse.js` | ~8 strings + formatPrice |
| `service-detail.js` | ~10 strings + formatPrice |
| `seeker-survey.js` | ~15 strings + CATEGORIES/URGENCY/AVAILABILITY |
| `suggestions.js` | ~8 strings + formatPrice |
| `create-service.js` | ~12 strings + CATEGORIES |
| `provider-survey.js` | ~12 strings + CATEGORIES/EXPERIENCE/AVAILABILITY |
| `main.css` | Add `.lang-picker` styles |

### New files:
- `frontend/src/i18n/i18n.js`
- `frontend/src/i18n/locales/en.json` (~120 keys)
- `frontend/src/i18n/locales/es.json`
- `frontend/src/i18n/locales/pt.json`

---

## Implementation Phases

### Phase 1: Foundation
1. Create `i18n.js` module
2. Create `en.json` with all keys extracted from codebase
3. Update `main.js` for async boot + lang picker
4. Add CSS for lang picker

### Phase 2: Migrate Pages (incremental, one file at a time)
5. `home.js` (simplest — proof of concept)
6. `login.js`, `register.js`, `forgot-password.js`, `not-found.js`
7. Extract shared `formatPrice()` utility
8. `browse.js`, `service-detail.js`, `suggestions.js`
9. `dashboard.js`
10. `create-service.js`, `provider-survey.js`, `seeker-survey.js`

### Phase 3: Add Languages
11. Create `es.json` (Spanish)
12. Create `pt.json` (Portuguese)
13. End-to-end testing

### Phase 4: Polish
14. Add `translateError()` to all catch blocks
15. Test string length issues (Spanish/Portuguese are ~10-30% longer)
16. Verify re-render correctly translates all visible content

---

## Considerations

- **Category values in DB:** Store normalized English keys (e.g. `"tutoring"`) as canonical identifiers. Only the display label is translated.
- **Async re-fetch on language switch:** Switching language re-triggers page handlers, which re-fetch data. Acceptable since language changes are rare.
- **Pluralization:** Minimal need. Can add `tp(key, count)` helper later if needed.
- **Backend suggestions reasons:** Map the known set of reason strings on the frontend, same as error translation.
