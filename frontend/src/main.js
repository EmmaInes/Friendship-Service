import { route, start } from './router.js';
import { isLoggedIn, getUser, clearAuth, api } from './api.js';
import { initI18n, t, getLocale, setLocale, getSupportedLocales } from './i18n/i18n.js';
import home from './pages/shared/home.js';
import login from './pages/shared/login.js';
import register from './pages/shared/register.js';
import notFound from './pages/shared/not-found.js';
import forgotPassword from './pages/shared/forgot-password.js';
import browse from './pages/seeker/browse.js';
import serviceDetail from './pages/seeker/service-detail.js';
import createService from './pages/provider/create-service.js';
import dashboard from './pages/shared/dashboard.js';
import providerSurvey from './pages/provider/provider-survey.js';
import seekerSurvey from './pages/seeker/seeker-survey.js';
import suggestions from './pages/seeker/suggestions.js';
import chat from './pages/shared/chat.js';

const FLAGS = { en: '\u{1F1FA}\u{1F1F8}', es: '\u{1F1E6}\u{1F1F7}', pt: '\u{1F1E7}\u{1F1F7}' };

function renderNav() {
  const nav = document.getElementById('nav');
  const user = getUser();
  const locale = getLocale();

  nav.innerHTML = `
    <div class="nav-inner">
      <a href="#/" class="nav-brand"><img src="/logo.svg" alt="Friendship&amp;Service" class="nav-logo" /></a>
      <div class="nav-links">
        ${isLoggedIn() ? `
          <a href="#/services">${t('nav.browse')}</a>
          <a href="#/suggestions">${t('nav.forYou')}</a>
          <a href="#/services/new">${t('nav.offer')}</a>
          <a href="#/survey/seeker">${t('nav.seekerSurvey')}</a>
          <a href="#/survey/provider">${t('nav.providerSurvey')}</a>
          <a href="#/dashboard" id="nav-dashboard">${t('nav.dashboard')}<span id="unread-badge" class="nav-badge" style="display:none"></span></a>
          <span class="nav-user">${user?.display_name || 'Account'}</span>
          <button id="logout-btn" class="btn btn-small">${t('nav.logOut')}</button>
        ` : `
          <a href="#/login">${t('nav.logIn')}</a>
          <a href="#/register">${t('nav.signUp')}</a>
        `}
        <select id="lang-picker" class="lang-picker">
          ${getSupportedLocales().map(loc => `
            <option value="${loc}" ${loc === locale ? 'selected' : ''}>${FLAGS[loc]} ${t('lang.' + loc)}</option>
          `).join('')}
        </select>
      </div>
    </div>
  `;
}

// Event delegation on nav (logout + language picker)
document.getElementById('nav').addEventListener('click', (e) => {
  if (e.target.id === 'logout-btn') {
    clearAuth();
    window.location.hash = '/login';
  }
});

document.getElementById('nav').addEventListener('change', (e) => {
  if (e.target.id === 'lang-picker') {
    setLocale(e.target.value);
  }
});

// Re-render nav on every hash change to reflect auth state + language
window.addEventListener('hashchange', renderNav);

// Register routes
route('/', home);
route('/login', login);
route('/register', register);
route('/forgot-password', forgotPassword);
route('/services', browse);
route('/services/new', createService);
route('/services/:id', serviceDetail);
route('/dashboard', dashboard);
route('/survey/provider', providerSurvey);
route('/survey/seeker', seekerSurvey);
route('/suggestions', suggestions);
route('/chat/:id', chat);
route('/404', notFound);

// Unread message badge polling
let unreadInterval = null;

async function pollUnread() {
  if (!isLoggedIn()) return;
  try {
    const data = await api.getUnreadCount();
    const badge = document.getElementById('unread-badge');
    if (badge) {
      if (data.count > 0) {
        badge.textContent = data.count;
        badge.style.display = '';
      } else {
        badge.style.display = 'none';
      }
    }
  } catch { /* silent */ }
}

function startUnreadPolling() {
  if (unreadInterval) clearInterval(unreadInterval);
  if (isLoggedIn()) {
    pollUnread();
    unreadInterval = setInterval(pollUnread, 30000);
  }
}

// Re-check polling on nav changes (login/logout)
window.addEventListener('hashchange', startUnreadPolling);

// Async boot — load translations before first render
async function boot() {
  await initI18n();
  const app = document.getElementById('app');
  renderNav();
  start(app);
  startUnreadPolling();
}
boot();
