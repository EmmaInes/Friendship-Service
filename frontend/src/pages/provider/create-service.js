import { api, isLoggedIn } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';
import { CATEGORIES } from '../../utils.js';

const PRICE_TYPES = ['negotiable', 'free', 'fixed', 'hourly'];

export default function createService(app) {
  if (!isLoggedIn()) {
    navigate('/login');
    return;
  }

  app.innerHTML = `
    <section>
      <h2>${t('createService.title')}</h2>
      <form id="service-form" class="service-form">
        <label>
          ${t('createService.titleLabel')}
          <input type="text" name="title" required maxlength="200" placeholder="${t('createService.titlePlaceholder')}" />
        </label>
        <label>
          ${t('createService.category')}
          <select name="category" required>
            <option value="">${t('createService.selectCategory')}</option>
            ${CATEGORIES.map(c => `<option value="${c}">${t('category.' + c)}</option>`).join('')}
          </select>
        </label>
        <label>
          ${t('createService.description')}
          <textarea name="description" required placeholder="${t('createService.descPlaceholder')}"></textarea>
        </label>
        <label>
          ${t('createService.pricing')}
          <select name="price_type">
            ${PRICE_TYPES.map(pt => `<option value="${pt}">${t('priceType.' + pt)}</option>`).join('')}
          </select>
        </label>
        <label id="price-label" style="display:none">
          ${t('createService.amount')}
          <input type="number" name="price" min="0" step="0.01" placeholder="0.00" />
        </label>
        <label>
          ${t('createService.location')}
          <input type="text" name="location" placeholder="${t('createService.locationPlaceholder')}" />
        </label>
        <p class="error-msg" id="service-error"></p>
        <button type="submit" class="btn btn-primary">${t('createService.submit')}</button>
      </form>
    </section>
  `;

  const form = document.getElementById('service-form');
  const priceType = form.price_type;
  const priceLabel = document.getElementById('price-label');

  priceType.addEventListener('change', () => {
    priceLabel.style.display =
      priceType.value === 'fixed' || priceType.value === 'hourly' ? '' : 'none';
  });

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const errorEl = document.getElementById('service-error');
    errorEl.textContent = '';

    const data = {
      title: form.title.value,
      description: form.description.value,
      category: form.category.value,
      price_type: form.price_type.value,
      location: form.location.value || undefined,
    };

    if (data.price_type === 'fixed' || data.price_type === 'hourly') {
      const dollars = parseFloat(form.price.value);
      if (isNaN(dollars) || dollars < 0) {
        errorEl.textContent = t('createService.invalidPrice');
        return;
      }
      data.price_cents = Math.round(dollars * 100);
    }

    try {
      await api.createService(data);
      navigate('/dashboard');
    } catch (err) {
      errorEl.textContent = translateError(err.error, 'createService.failed');
    }
  });
}
