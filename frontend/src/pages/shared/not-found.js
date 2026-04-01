import { t } from '../../i18n/i18n.js';

export default function notFound(app) {
  app.innerHTML = `
    <section class="not-found">
      <h2>${t('notFound.title')}</h2>
      <p>${t('notFound.message')}</p>
      <a href="#/" class="btn btn-primary">${t('notFound.goHome')}</a>
    </section>
  `;
}
