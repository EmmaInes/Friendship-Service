import { route, start } from './router.js';
import { isLoggedIn, getUser, clearAuth } from './api.js';
import home from './pages/shared/home.js';
import login from './pages/shared/login.js';
import register from './pages/shared/register.js';
import notFound from './pages/shared/not-found.js';
import browse from './pages/seeker/browse.js';
import serviceDetail from './pages/seeker/service-detail.js';
import createService from './pages/provider/create-service.js';
import dashboard from './pages/shared/dashboard.js';

function renderNav() {
  const nav = document.getElementById('nav');
  const user = getUser();

  nav.innerHTML = `
    <div class="nav-inner">
      <a href="#/" class="nav-brand">Friendship&amp;Service</a>
      <div class="nav-links">
        ${isLoggedIn() ? `
          <a href="#/services">Browse</a>
          <a href="#/services/new">Offer</a>
          <a href="#/dashboard">Dashboard</a>
          <span class="nav-user">${user?.display_name || 'Account'}</span>
          <button id="logout-btn" class="btn btn-small">Log Out</button>
        ` : `
          <a href="#/login">Log In</a>
          <a href="#/register">Sign Up</a>
        `}
      </div>
    </div>
  `;

  const logoutBtn = document.getElementById('logout-btn');
  if (logoutBtn) {
    logoutBtn.addEventListener('click', () => {
      clearAuth();
      window.location.hash = '/';
    });
  }
}

// Re-render nav on every hash change to reflect auth state
window.addEventListener('hashchange', renderNav);

// Register routes
route('/', home);
route('/login', login);
route('/register', register);
route('/services', browse);
route('/services/new', createService);
route('/services/:id', serviceDetail);
route('/dashboard', dashboard);
route('/404', notFound);

// Boot
const app = document.getElementById('app');
renderNav();
start(app);
