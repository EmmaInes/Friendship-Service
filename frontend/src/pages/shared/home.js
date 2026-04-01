import { isLoggedIn, getUser } from '../../api.js';
import { t } from '../../i18n/i18n.js';

export default function home(app) {
  const user = getUser();

  app.innerHTML = `
    <section class="hero">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="hero-logo" />
      <h1>Friendship &amp; Service</h1>
      <p>${t('home.tagline')}</p>
      ${isLoggedIn() ? `
        <p>${t('home.welcomeBack', { name: user.display_name })}</p>
        <nav class="hero-actions">
          <a href="#/services" class="btn btn-primary">${t('home.browseServices')}</a>
          <a href="#/services/new" class="btn btn-secondary">${t('home.offerService')}</a>
        </nav>
      ` : `
        <nav class="hero-actions">
          <a href="#/login" class="btn btn-primary">${t('nav.logIn')}</a>
          <a href="#/register" class="btn btn-secondary">${t('nav.signUp')}</a>
        </nav>
      `}
    </section>
  `;
}
