import { api, isLoggedIn } from '../../api.js';
import { t } from '../../i18n/i18n.js';
import { formatPrice } from '../../utils.js';
import { renderStars } from '../../components/star-rating.js';

export default async function browse(app) {
  app.innerHTML = `<p>${t('browse.loading')}</p>`;

  try {
    const services = await api.getServices();

    if (services.length === 0) {
      app.innerHTML = `
        <section>
          <h2>${t('browse.title')}</h2>
          <p class="empty-state">${t('browse.empty')} <a href="#/services/new">${t('browse.offerOne')}</a>!</p>
        </section>
      `;
      return;
    }

    app.innerHTML = `
      <section>
        <h2>${t('browse.title')}</h2>
        <div class="services-grid">
          ${services.map(s => `
            <div class="service-card">
              <span class="category">${t('category.' + s.category)}</span>
              <h3>${s.title}</h3>
              <p>${s.description.length > 120 ? s.description.slice(0, 120) + '...' : s.description}</p>
              <p class="price">${formatPrice(s.price_cents, s.price_type)}</p>
              <p style="font-size:0.8rem;color:var(--color-text-muted)">${t('browse.by', { name: s.provider_name })}</p>
              <p style="font-size:0.85rem">${renderStars(s.avg_rating, s.review_count)}</p>
              ${isLoggedIn()
                ? `<a href="#/services/${s.id}" class="btn btn-primary" style="margin-top:var(--space-sm)">${t('browse.viewDetails')}</a>`
                : `<a href="#/login" class="btn btn-primary" style="margin-top:var(--space-sm)">${t('browse.loginToRequest')}</a>`
              }
            </div>
          `).join('')}
        </div>
      </section>
    `;
  } catch {
    app.innerHTML = `<p class="error-msg">${t('browse.loadFailed')}</p>`;
  }
}
