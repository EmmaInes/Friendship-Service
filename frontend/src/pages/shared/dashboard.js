import { api, isLoggedIn, getUser } from '../../api.js';
import { navigate } from '../../router.js';
import { t } from '../../i18n/i18n.js';
import { formatPrice } from '../../utils.js';

export default async function dashboard(app) {
  if (!isLoggedIn()) {
    navigate('/login');
    return;
  }

  const user = getUser();
  app.innerHTML = `<p>${t('dashboard.loading')}</p>`;

  try {
    const [services, requests] = await Promise.all([
      api.getMyServices(),
      api.getMyRequests(),
    ]);

    app.innerHTML = `
      <section class="dashboard">
        <h2>${t('dashboard.title')}</h2>
        <p style="color:var(--color-text-muted);margin-bottom:var(--space-xl)">${t('dashboard.welcome', { name: user.display_name })}</p>

        <section>
          <h3>${t('dashboard.myServices', { count: services.length })}</h3>
          ${services.length === 0
            ? `<p class="empty-state">${t('dashboard.noServices')} <a href="#/services/new">${t('dashboard.createOne')}</a></p>`
            : `<div class="services-grid">${services.map(s => `
                <div class="service-card">
                  <span class="category">${t('category.' + s.category)}</span>
                  <h3>${s.title}</h3>
                  <p class="price">${formatPrice(s.price_cents, s.price_type)}</p>
                  <p style="font-size:0.8rem;color:var(--color-text-muted)">${s.is_active ? t('common.active') : t('common.inactive')}</p>
                </div>
              `).join('')}</div>`
          }
        </section>

        <section>
          <h3>${t('dashboard.requests', { count: requests.length })}</h3>
          ${requests.length === 0
            ? `<p class="empty-state">${t('dashboard.noRequests')}</p>`
            : `<table style="width:100%;border-collapse:collapse;font-size:0.9rem">
                <thead>
                  <tr style="text-align:left;border-bottom:2px solid var(--color-border)">
                    <th style="padding:var(--space-sm)">${t('dashboard.colService')}</th>
                    <th style="padding:var(--space-sm)">${t('dashboard.colMessage')}</th>
                    <th style="padding:var(--space-sm)">${t('dashboard.colStatus')}</th>
                    <th style="padding:var(--space-sm)">${t('dashboard.colDate')}</th>
                  </tr>
                </thead>
                <tbody>
                  ${requests.map(r => `
                    <tr style="border-bottom:1px solid var(--color-border)">
                      <td style="padding:var(--space-sm)">${r.service_title}</td>
                      <td style="padding:var(--space-sm)">${r.message || '-'}</td>
                      <td style="padding:var(--space-sm)">${t('status.' + r.status)}</td>
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
    app.innerHTML = `<p class="error-msg">${t('dashboard.loadFailed')}</p>`;
  }
}
