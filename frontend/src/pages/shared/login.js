import { api, setAuth } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';

export default function login(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>${t('login.title')}</h2>
      <form id="login-form" class="auth-form">
        <label>
          ${t('login.email')}
          <input type="email" name="email" required autocomplete="email" />
        </label>
        <label>
          ${t('login.password')}
          <input type="password" name="password" required autocomplete="current-password" />
        </label>
        <p class="error-msg" id="login-error"></p>
        <button type="submit" class="btn btn-primary">${t('login.submit')}</button>
      </form>
      <p class="auth-switch"><a href="#/forgot-password">${t('login.forgotPassword')}</a></p>
      <p class="auth-switch">${t('login.noAccount')} <a href="#/register">${t('login.signUpLink')}</a></p>
    </section>
  `;

  const form = document.getElementById('login-form');
  const errorEl = document.getElementById('login-error');

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.textContent = '';

    const data = Object.fromEntries(new FormData(form));

    try {
      const res = await api.login(data);
      setAuth(res.token, res.user);
      navigate('/');
    } catch (err) {
      errorEl.textContent = translateError(err.error, 'login.failed');
    }
  });
}
