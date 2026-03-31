import { api, isLoggedIn } from '../../api.js';

function formatPrice(cents, type) {
  if (type === 'free') return 'Free';
  if (type === 'negotiable') return 'Negotiable';
  if (cents == null) return type;
  const dollars = (cents / 100).toFixed(2);
  return `$${dollars}${type === 'hourly' ? '/hr' : ''}`;
}

export default async function browse(app) {
  app.innerHTML = '<p>Loading services...</p>';

  try {
    const services = await api.getServices();

    if (services.length === 0) {
      app.innerHTML = `
        <section>
          <h2>Browse Services</h2>
          <p class="empty-state">No services available yet. Be the first to <a href="#/services/new">offer one</a>!</p>
        </section>
      `;
      return;
    }

    app.innerHTML = `
      <section>
        <h2>Browse Services</h2>
        <div class="services-grid">
          ${services.map(s => `
            <div class="service-card">
              <span class="category">${s.category}</span>
              <h3>${s.title}</h3>
              <p>${s.description.length > 120 ? s.description.slice(0, 120) + '...' : s.description}</p>
              <p class="price">${formatPrice(s.price_cents, s.price_type)}</p>
              <p style="font-size:0.8rem;color:var(--color-text-muted)">by ${s.provider_name}</p>
              ${isLoggedIn()
                ? `<a href="#/services/${s.id}" class="btn btn-primary" style="margin-top:var(--space-sm)">View Details</a>`
                : `<a href="#/login" class="btn btn-primary" style="margin-top:var(--space-sm)">Log in to request</a>`
              }
            </div>
          `).join('')}
        </div>
      </section>
    `;
  } catch {
    app.innerHTML = '<p class="error-msg">Failed to load services. Is the backend running?</p>';
  }
}
