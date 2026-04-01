import { api, isLoggedIn } from '../../api.js';
import { navigate } from '../../router.js';
import { t } from '../../i18n/i18n.js';
import { formatPrice } from '../../utils.js';
import { renderStars } from '../../components/star-rating.js';

export default async function suggestions(app) {
  if (!isLoggedIn()) { navigate('/login'); return; }

  app.innerHTML = `<p>${t('suggestions.loading')}</p>`;

  try {
    const data = await api.getSuggestions();

    if (data.message && data.suggestions.length === 0) {
      app.innerHTML = `
        <section class="suggestions-page">
          <h2>${t('suggestions.title')}</h2>
          <div class="suggestion-empty">
            <p>${t('suggestions.completeSurvey')}</p>
            <a href="#/survey/seeker" class="btn btn-primary">${t('suggestions.takeSurvey')}</a>
          </div>
        </section>
      `;
      return;
    }

    if (data.suggestions.length === 0) {
      app.innerHTML = `
        <section class="suggestions-page">
          <h2>${t('suggestions.title')}</h2>
          <p class="empty-state">${t('suggestions.empty')}</p>
          <a href="#/services" class="btn btn-secondary" style="margin-top:var(--space-md)">${t('suggestions.browseAll')}</a>
        </section>
      `;
      return;
    }

    app.innerHTML = `
      <section class="suggestions-page">
        <h2>${t('suggestions.title')}</h2>
        <p style="color:var(--color-text-muted);margin-bottom:var(--space-lg)">${t('suggestions.basedOn')}</p>
        <div class="services-grid">
          ${data.suggestions.map(({ service: s, score, reasons }) => `
            <div class="service-card suggestion-card">
              <div class="suggestion-score" title="${t('suggestions.matchScore', { score })}">
                ${'★'.repeat(Math.min(score, 5))}${'☆'.repeat(Math.max(0, 5 - score))}
              </div>
              <span class="category">${t('category.' + s.category)}</span>
              <h3>${s.title}</h3>
              <p>${s.description.length > 120 ? s.description.slice(0, 120) + '...' : s.description}</p>
              <p class="price">${formatPrice(s.price_cents, s.price_type)}</p>
              <p style="font-size:0.8rem;color:var(--color-text-muted)">${t('browse.by', { name: s.provider_name })}</p>
              <p style="font-size:0.85rem">${renderStars(s.avg_rating, s.review_count)}</p>
              <ul class="suggestion-reasons">
                ${reasons.map(r => `<li>${r}</li>`).join('')}
              </ul>
              <a href="#/services/${s.id}" class="btn btn-primary" style="margin-top:var(--space-sm)">${t('browse.viewDetails')}</a>
            </div>
          `).join('')}
        </div>
      </section>
    `;
  } catch {
    app.innerHTML = `<p class="error-msg">${t('suggestions.loadFailed')}</p>`;
  }
}
