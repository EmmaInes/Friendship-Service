import { api, setAuth } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';

export default function register(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>${t('register.title')}</h2>
      <form id="register-form" class="auth-form">
        <label>
          ${t('register.displayName')}
          <input type="text" name="display_name" required minlength="1" maxlength="100" />
        </label>
        <label>
          ${t('register.username')}
          <input type="text" name="username" required minlength="3" maxlength="30" autocomplete="username" />
        </label>
        <label>
          ${t('register.email')}
          <input type="email" name="email" required autocomplete="email" />
        </label>
        <label>
          ${t('register.password')}
          <input type="password" name="password" required minlength="8" autocomplete="new-password" />
        </label>
        <p class="error-msg" id="register-error"></p>
        <button type="submit" class="btn btn-primary">${t('register.submit')}</button>
      </form>
      <p class="auth-switch">${t('register.hasAccount')} <a href="#/login">${t('register.loginLink')}</a></p>
    </section>
  `;

  const form = document.getElementById('register-form');
  const errorEl = document.getElementById('register-error');

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.textContent = '';

    const data = Object.fromEntries(new FormData(form));

    try {
      const res = await api.register(data);
      setAuth(res.token, res.user);
      navigate('/');
    } catch (err) {
      errorEl.textContent = translateError(err.error, 'register.failed');
    }
  });
}
