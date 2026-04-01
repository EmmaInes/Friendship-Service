import { api, isLoggedIn, getUser } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';
import { formatPrice } from '../../utils.js';

export default async function serviceDetail(app, id) {
  app.innerHTML = `<p>${t('common.loading')}</p>`;

  try {
    const s = await api.getService(id);
    const user = getUser();
    const isOwner = user && user.id === s.provider_id;

    app.innerHTML = `
      <section style="max-width:700px;margin:0 auto">
        <a href="#/services" style="color:var(--color-primary);font-size:0.9rem">${t('detail.backToServices')}</a>
        <span class="category" style="margin-top:var(--space-md);display:inline-block">${t('category.' + s.category)}</span>
        <h2 style="margin:var(--space-sm) 0">${s.title}</h2>
        <p style="color:var(--color-text-muted);font-size:0.9rem">${t('detail.offeredBy', { name: s.provider_name, username: s.provider_username })}</p>
        <p style="margin:var(--space-md) 0">${s.description}</p>
        <p class="price" style="font-size:1.2rem">${formatPrice(s.price_cents, s.price_type)}</p>
        ${s.location ? `<p style="font-size:0.9rem;color:var(--color-text-muted)">${t('detail.location', { location: s.location })}</p>` : ''}

        ${isLoggedIn() && !isOwner ? `
          <div style="margin-top:var(--space-xl);padding-top:var(--space-lg);border-top:1px solid var(--color-border)">
            <h3>${t('detail.requestTitle')}</h3>
            <form id="request-form" style="margin-top:var(--space-md)">
              <label style="display:flex;flex-direction:column;gap:var(--space-xs)">
                ${t('detail.messageLabel')}
                <textarea name="message" rows="3" placeholder="${t('detail.messagePlaceholder')}" style="padding:var(--space-sm);border:1px solid var(--color-border);border-radius:var(--radius-sm);font-family:inherit"></textarea>
              </label>
              <p class="error-msg" id="request-error"></p>
              <button type="submit" class="btn btn-primary" style="margin-top:var(--space-sm)">${t('detail.sendRequest')}</button>
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
          errorEl.textContent = translateError(err.error, 'detail.requestFailed');
        }
      });
    }
  } catch {
    app.innerHTML = `<p class="error-msg">${t('detail.notFound')}</p>`;
  }
}
