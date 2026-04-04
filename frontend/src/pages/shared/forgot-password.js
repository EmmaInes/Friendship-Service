import { t } from '../../i18n/i18n.js';

export default function forgotPassword(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>${t('forgot.title')}</h2>
      <p style="text-align:center;color:var(--color-text-muted);margin-bottom:var(--space-lg)">
        ${t('forgot.notNeeded')}
      </p>
      <div style="text-align:center">
        <a href="#/login" class="btn btn-primary">${t('forgot.goToLogin')}</a>
      </div>
    </section>
  `;
}
