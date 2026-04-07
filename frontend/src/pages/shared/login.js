import { api, setAuth } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';
import { GOOGLE_CLIENT_ID } from '../../config.js';

export default function login(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>${t('login.title')}</h2>

      <form class="auth-form" id="login-form">
        <label>
          ${t('login.email')}
          <input type="email" id="login-email" required />
        </label>
        <label>
          ${t('login.password')}
          <div class="password-wrap">
            <input type="password" id="login-password" required />
            <button type="button" class="password-toggle" aria-label="${t('auth.showPassword')}">${t('auth.show')}</button>
          </div>
        </label>
        <button type="submit" class="btn btn-primary">${t('login.submit')}</button>
      </form>

      <p class="auth-link" style="text-align:right;margin-top:var(--space-xs)">
        <a href="#/forgot-password">${t('login.forgotPassword')}</a>
      </p>

      <div class="auth-divider"><span>${t('auth.or')}</span></div>

      <div id="g_id_signin" class="google-btn-wrap"></div>

      <p class="error-msg" id="login-error" style="text-align:center;margin-top:var(--space-md)"></p>

      <p class="auth-switch">${t('login.noAccount')} <a href="#/register">${t('login.signUpLink')}</a></p>
    </section>
  `;

  const errorEl = document.getElementById('login-error');
  const form = document.getElementById('login-form');

  form.querySelectorAll('.password-toggle').forEach(btn => {
    btn.addEventListener('click', () => {
      const input = btn.previousElementSibling;
      const visible = input.type === 'text';
      input.type = visible ? 'password' : 'text';
      btn.textContent = visible ? t('auth.show') : t('auth.hide');
    });
  });

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.textContent = '';
    const email = document.getElementById('login-email').value.trim();
    const password = document.getElementById('login-password').value;

    try {
      const res = await api.login({ email, password });
      setAuth(res.token, res.user);
      navigate('/');
    } catch (err) {
      errorEl.textContent = translateError(err.error, 'login.failed');
    }
  });

  async function handleCredentialResponse(response) {
    errorEl.textContent = '';
    try {
      const res = await api.googleLogin({ credential: response.credential });
      setAuth(res.token, res.user);
      navigate('/');
    } catch (err) {
      errorEl.textContent = translateError(err.error, 'login.googleFailed');
    }
  }

  function initGSI() {
    if (typeof google === 'undefined' || !google.accounts) {
      setTimeout(initGSI, 200);
      return;
    }
    google.accounts.id.initialize({
      client_id: GOOGLE_CLIENT_ID,
      callback: handleCredentialResponse,
    });
    google.accounts.id.renderButton(
      document.getElementById('g_id_signin'),
      { theme: 'outline', size: 'large', text: 'signin_with', width: 320 }
    );
  }

  initGSI();
}
