import { api, isLoggedIn } from '../../api.js';
import { navigate } from '../../router.js';

function formatPrice(cents, type) {
  if (type === 'free') return 'Free';
  if (type === 'negotiable') return 'Negotiable';
  if (cents == null) return type;
  const dollars = (cents / 100).toFixed(2);
  return `$${dollars}${type === 'hourly' ? '/hr' : ''}`;
}

export default async function suggestions(app) {
  if (!isLoggedIn()) { navigate('/login'); return; }

  app.innerHTML = '<p>Loading suggestions...</p>';

  try {
    const data = await api.getSuggestions();

    if (data.message && data.suggestions.length === 0) {
      app.innerHTML = `
        <section class="suggestions-page">
          <h2>Suggested For You</h2>
          <div class="suggestion-empty">
            <p>${data.message}</p>
            <a href="#/survey/seeker" class="btn btn-primary">Take Seeker Survey</a>
          </div>
        </section>
      `;
      return;
    }

    if (data.suggestions.length === 0) {
      app.innerHTML = `
        <section class="suggestions-page">
          <h2>Suggested For You</h2>
          <p class="empty-state">No matching services found yet. Check back as more providers join!</p>
          <a href="#/services" class="btn btn-secondary" style="margin-top:var(--space-md)">Browse All Services</a>
        </section>
      `;
      return;
    }

    app.innerHTML = `
      <section class="suggestions-page">
        <h2>Suggested For You</h2>
        <p style="color:var(--color-text-muted);margin-bottom:var(--space-lg)">Based on your seeker survey preferences</p>
        <div class="services-grid">
          ${data.suggestions.map(({ service: s, score, reasons }) => `
            <div class="service-card suggestion-card">
              <div class="suggestion-score" title="Match score: ${score}">
                ${'★'.repeat(Math.min(score, 5))}${'☆'.repeat(Math.max(0, 5 - score))}
              </div>
              <span class="category">${s.category}</span>
              <h3>${s.title}</h3>
              <p>${s.description.length > 120 ? s.description.slice(0, 120) + '...' : s.description}</p>
              <p class="price">${formatPrice(s.price_cents, s.price_type)}</p>
              <p style="font-size:0.8rem;color:var(--color-text-muted)">by ${s.provider_name}</p>
              <ul class="suggestion-reasons">
                ${reasons.map(r => `<li>${r}</li>`).join('')}
              </ul>
              <a href="#/services/${s.id}" class="btn btn-primary" style="margin-top:var(--space-sm)">View Details</a>
            </div>
          `).join('')}
        </div>
      </section>
    `;
  } catch {
    app.innerHTML = '<p class="error-msg">Failed to load suggestions.</p>';
  }
}
