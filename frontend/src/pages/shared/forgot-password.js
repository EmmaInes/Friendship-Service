import { api } from '../../api.js';
import { t, translateError } from '../../i18n/i18n.js';

export default function forgotPassword(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>${t('forgot.title')}</h2>
      <p style="text-align:center;color:var(--color-text-muted);margin-bottom:var(--space-lg)">
        ${t('forgot.instructions')}
      </p>

      <form class="auth-form" id="forgot-form">
        <label>
          ${t('forgot.email')}
          <input type="email" id="forgot-email" required />
        </label>
        <label>
          ${t('forgot.username')}
          <input type="text" id="forgot-username" required />
        </label>
        <label>
          ${t('forgot.newPassword')}
          <input type="password" id="forgot-new-password" required minlength="8" />
        </label>
        <label>
          ${t('forgot.confirmPassword')}
          <input type="password" id="forgot-confirm-password" required minlength="8" />
        </label>
        <button type="submit" class="btn btn-primary">${t('forgot.submit')}</button>
      </form>

      <p class="error-msg" id="forgot-error" style="text-align:center;margin-top:var(--space-md)"></p>
      <p class="success-msg" id="forgot-success" style="text-align:center;margin-top:var(--space-md);color:var(--color-success,green)"></p>

      <p class="auth-switch"><a href="#/login">${t('forgot.backToLogin')}</a></p>
    </section>
  `;

  const errorEl = document.getElementById('forgot-error');
  const successEl = document.getElementById('forgot-success');
  const form = document.getElementById('forgot-form');

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.textContent = '';
    successEl.textContent = '';

    const email = document.getElementById('forgot-email').value.trim();
    const username = document.getElementById('forgot-username').value.trim();
    const newPassword = document.getElementById('forgot-new-password').value;
    const confirmPassword = document.getElementById('forgot-confirm-password').value;

    if (newPassword !== confirmPassword) {
      errorEl.textContent = t('forgot.passwordsMismatch');
      return;
    }

    try {
      await api.resetPassword({ email, username, new_password: newPassword });
      successEl.textContent = t('forgot.success');
      form.reset();
      setTimeout(() => { window.location.hash = '#/login'; }, 2000);
    } catch (err) {
      errorEl.textContent = translateError(err.error, 'forgot.failed');
    }
  });
}
