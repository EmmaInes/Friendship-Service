import { api, setAuth } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';
import { GOOGLE_CLIENT_ID } from '../../config.js';

export default function register(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>${t('register.title')}</h2>

      <form class="auth-form" id="register-form">
        <label>
          ${t('register.displayName')}
          <input type="text" id="reg-display-name" required maxlength="100" />
        </label>
        <label>
          ${t('register.username')}
          <input type="text" id="reg-username" required minlength="3" maxlength="30" />
        </label>
        <label>
          ${t('register.email')}
          <input type="email" id="reg-email" required />
        </label>
        <label>
          ${t('register.password')}
          <div class="password-wrap">
            <input type="password" id="reg-password" required minlength="8" />
            <button type="button" class="password-toggle" aria-label="${t('auth.showPassword')}">${t('auth.show')}</button>
          </div>
          <small style="color:var(--color-text-muted)">${t('register.passwordHint')}</small>
        </label>
        <label>
          ${t('register.confirmPassword')}
          <div class="password-wrap">
            <input type="password" id="reg-confirm-password" required minlength="8" />
            <button type="button" class="password-toggle" aria-label="${t('auth.showPassword')}">${t('auth.show')}</button>
          </div>
        </label>
        <button type="submit" class="btn btn-primary">${t('register.submit')}</button>
      </form>

      <div class="auth-divider"><span>${t('auth.or')}</span></div>

      <div id="g_id_signup" class="google-btn-wrap"></div>

      <p class="error-msg" id="register-error" style="text-align:center;margin-top:var(--space-md)"></p>

      <p class="auth-switch">${t('register.hasAccount')} <a href="#/login">${t('register.loginLink')}</a></p>
    </section>
  `;

  const errorEl = document.getElementById('register-error');
  const form = document.getElementById('register-form');

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

    const display_name = document.getElementById('reg-display-name').value.trim();
    const username = document.getElementById('reg-username').value.trim();
    const email = document.getElementById('reg-email').value.trim();
    const password = document.getElementById('reg-password').value;
    const confirmPassword = document.getElementById('reg-confirm-password').value;

    if (password !== confirmPassword) {
      errorEl.textContent = t('register.passwordsMismatch');
      return;
    }

    try {
      const res = await api.register({ email, username, password, display_name });
      setAuth(res.token, res.user);
      navigate('/');
    } catch (err) {
      errorEl.textContent = translateError(err.error, 'register.failed');
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
      document.getElementById('g_id_signup'),
      { theme: 'outline', size: 'large', text: 'signup_with', width: 320 }
    );
  }

  initGSI();
}
