import { api, isLoggedIn, getUser } from '../../api.js';
import { navigate } from '../../router.js';

function formatPrice(cents, type) {
  if (type === 'free') return 'Free';
  if (type === 'negotiable') return 'Negotiable';
  if (cents == null) return type;
  const dollars = (cents / 100).toFixed(2);
  return `$${dollars}${type === 'hourly' ? '/hr' : ''}`;
}

const STATUS_LABELS = {
  pending: 'Pending',
  accepted: 'Accepted',
  declined: 'Declined',
  completed: 'Completed',
  cancelled: 'Cancelled',
};

export default async function dashboard(app) {
  if (!isLoggedIn()) {
    navigate('/login');
    return;
  }

  const user = getUser();
  app.innerHTML = '<p>Loading dashboard...</p>';

  try {
    const [services, requests] = await Promise.all([
      api.getMyServices(),
      api.getMyRequests(),
    ]);

    app.innerHTML = `
      <section class="dashboard">
        <h2>Dashboard</h2>
        <p style="color:var(--color-text-muted);margin-bottom:var(--space-xl)">Welcome, ${user.display_name}</p>

        <section>
          <h3>My Services (${services.length})</h3>
          ${services.length === 0
            ? '<p class="empty-state">You haven\'t offered any services yet. <a href="#/services/new">Create one</a></p>'
            : `<div class="services-grid">${services.map(s => `
                <div class="service-card">
                  <span class="category">${s.category}</span>
                  <h3>${s.title}</h3>
                  <p class="price">${formatPrice(s.price_cents, s.price_type)}</p>
                  <p style="font-size:0.8rem;color:var(--color-text-muted)">${s.is_active ? 'Active' : 'Inactive'}</p>
                </div>
              `).join('')}</div>`
          }
        </section>

        <section>
          <h3>Requests (${requests.length})</h3>
          ${requests.length === 0
            ? '<p class="empty-state">No requests yet.</p>'
            : `<table style="width:100%;border-collapse:collapse;font-size:0.9rem">
                <thead>
                  <tr style="text-align:left;border-bottom:2px solid var(--color-border)">
                    <th style="padding:var(--space-sm)">Service</th>
                    <th style="padding:var(--space-sm)">Message</th>
                    <th style="padding:var(--space-sm)">Status</th>
                    <th style="padding:var(--space-sm)">Date</th>
                  </tr>
                </thead>
                <tbody>
                  ${requests.map(r => `
                    <tr style="border-bottom:1px solid var(--color-border)">
                      <td style="padding:var(--space-sm)">${r.service_title}</td>
                      <td style="padding:var(--space-sm)">${r.message || '-'}</td>
                      <td style="padding:var(--space-sm)">${STATUS_LABELS[r.status] || r.status}</td>
                      <td style="padding:var(--space-sm)">${new Date(r.created_at).toLocaleDateString()}</td>
                    </tr>
                  `).join('')}
                </tbody>
              </table>`
          }
        </section>
      </section>
    `;
  } catch {
    app.innerHTML = '<p class="error-msg">Failed to load dashboard.</p>';
  }
}
