import { api, isLoggedIn, getUser } from '../../api.js';
import { navigate } from '../../router.js';

function formatPrice(cents, type) {
  if (type === 'free') return 'Free';
  if (type === 'negotiable') return 'Negotiable';
  if (cents == null) return type;
  const dollars = (cents / 100).toFixed(2);
  return `$${dollars}${type === 'hourly' ? '/hr' : ''}`;
}

export default async function serviceDetail(app, id) {
  app.innerHTML = '<p>Loading...</p>';

  try {
    const s = await api.getService(id);
    const user = getUser();
    const isOwner = user && user.id === s.provider_id;

    app.innerHTML = `
      <section style="max-width:700px;margin:0 auto">
        <a href="#/services" style="color:var(--color-primary);font-size:0.9rem">&larr; Back to services</a>
        <span class="category" style="margin-top:var(--space-md);display:inline-block">${s.category}</span>
        <h2 style="margin:var(--space-sm) 0">${s.title}</h2>
        <p style="color:var(--color-text-muted);font-size:0.9rem">Offered by <strong>${s.provider_name}</strong> (@${s.provider_username})</p>
        <p style="margin:var(--space-md) 0">${s.description}</p>
        <p class="price" style="font-size:1.2rem">${formatPrice(s.price_cents, s.price_type)}</p>
        ${s.location ? `<p style="font-size:0.9rem;color:var(--color-text-muted)">Location: ${s.location}</p>` : ''}

        ${isLoggedIn() && !isOwner ? `
          <div style="margin-top:var(--space-xl);padding-top:var(--space-lg);border-top:1px solid var(--color-border)">
            <h3>Request this Service</h3>
            <form id="request-form" style="margin-top:var(--space-md)">
              <label style="display:flex;flex-direction:column;gap:var(--space-xs)">
                Message (optional)
                <textarea name="message" rows="3" placeholder="Describe what you need..." style="padding:var(--space-sm);border:1px solid var(--color-border);border-radius:var(--radius-sm);font-family:inherit"></textarea>
              </label>
              <p class="error-msg" id="request-error"></p>
              <button type="submit" class="btn btn-primary" style="margin-top:var(--space-sm)">Send Request</button>
            </form>
          </div>
        ` : ''}
      </section>
    `;

    const form = document.getElementById('request-form');
    if (form) {
      form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const errorEl = document.getElementById('request-error');
        errorEl.textContent = '';

        const message = form.message.value;
        try {
          await api.requestService(id, { message });
          navigate('/dashboard');
        } catch (err) {
          errorEl.textContent = err.error || 'Failed to send request';
        }
      });
    }
  } catch {
    app.innerHTML = '<p class="error-msg">Service not found.</p>';
  }
}
