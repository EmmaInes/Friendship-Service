import { api, isLoggedIn } from '../../api.js';
import { navigate } from '../../router.js';
import { t } from '../../i18n/i18n.js';
import { CATEGORIES } from '../../utils.js';

const AVAILABILITY_KEYS = ['weekdays', 'weekends', 'evenings', 'flexible'];
const URGENCY_KEYS = ['urgent', 'this_week', 'this_month', 'flexible'];

export default async function seekerSurvey(app) {
  if (!isLoggedIn()) { navigate('/login'); return; }

  app.innerHTML = `<p>${t('common.loading')}</p>`;

  let existing = null;
  try {
    const surveys = await api.getMySurveys();
    existing = surveys.find(s => s.survey_type === 'seeker');
  } catch { /* first time */ }

  const selectedCats = existing?.categories || [];
  const budgetMin = existing?.budget_min != null ? (existing.budget_min / 100).toFixed(2) : '';
  const budgetMax = existing?.budget_max != null ? (existing.budget_max / 100).toFixed(2) : '';

  app.innerHTML = `
    <section class="survey-page">
      <h2>${t('seekerSurvey.title')}</h2>
      <p class="survey-intro">${t('seekerSurvey.intro')}</p>
      <form id="seeker-survey-form" class="service-form">
        <fieldset>
          <legend>${t('seekerSurvey.categoriesLegend')}</legend>
          <div class="checkbox-grid">
            ${CATEGORIES.map(c => `
              <label class="checkbox-label">
                <input type="checkbox" name="categories" value="${c}" ${selectedCats.includes(c) ? 'checked' : ''} />
                ${t('category.' + c)}
              </label>
            `).join('')}
          </div>
        </fieldset>

        <fieldset>
          <legend>${t('seekerSurvey.budgetLegend')}</legend>
          <div class="inline-fields">
            <label>
              ${t('seekerSurvey.budgetMin')}
              <input type="number" name="budget_min" min="0" step="0.01" placeholder="0.00" value="${budgetMin}" />
            </label>
            <label>
              ${t('seekerSurvey.budgetMax')}
              <input type="number" name="budget_max" min="0" step="0.01" placeholder="100.00" value="${budgetMax}" />
            </label>
          </div>
        </fieldset>

        <label>
          ${t('seekerSurvey.urgency')}
          <select name="urgency">
            ${URGENCY_KEYS.map(u => `<option value="${u}" ${existing?.urgency === u ? 'selected' : ''}>${t('urgency.' + u)}</option>`).join('')}
          </select>
        </label>

        <label>
          ${t('seekerSurvey.availability')}
          <select name="availability">
            ${AVAILABILITY_KEYS.map(a => `<option value="${a}" ${existing?.availability === a ? 'selected' : ''}>${t('availability.' + a)}</option>`).join('')}
          </select>
        </label>

        <label>
          ${t('seekerSurvey.location')}
          <input type="text" name="location_preference" placeholder="${t('seekerSurvey.locationPlaceholder')}" value="${existing?.location_preference || ''}" />
        </label>

        <label>
          ${t('seekerSurvey.describe')}
          <textarea name="description" rows="4" placeholder="${t('seekerSurvey.describePlaceholder')}">${existing?.description || ''}</textarea>
        </label>

        <p class="error-msg" id="survey-error"></p>
        <p class="success-msg" id="survey-success"></p>
        <button type="submit" class="btn btn-primary">${existing ? t('seekerSurvey.update') : t('seekerSurvey.submit')}</button>
      </form>
    </section>
  `;

  const form = document.getElementById('seeker-survey-form');
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const errorEl = document.getElementById('survey-error');
    const successEl = document.getElementById('survey-success');
    errorEl.textContent = '';
    successEl.textContent = '';

    const checked = [...form.querySelectorAll('input[name="categories"]:checked')].map(el => el.value);
    if (checked.length === 0) {
      errorEl.textContent = t('seekerSurvey.selectOne');
      return;
    }

    const data = {
      survey_type: 'seeker',
      categories: checked,
      availability: form.availability.value,
      location_preference: form.location_preference.value || undefined,
      urgency: form.urgency.value,
      description: form.description.value || undefined,
    };

    const minVal = parseFloat(form.budget_min.value);
    const maxVal = parseFloat(form.budget_max.value);
    if (!isNaN(minVal)) data.budget_min = Math.round(minVal * 100);
    if (!isNaN(maxVal)) data.budget_max = Math.round(maxVal * 100);

    try {
      await api.saveSurvey(data);
      successEl.textContent = t('seekerSurvey.saved');
    } catch (err) {
      errorEl.textContent = err.error || t('seekerSurvey.failed');
    }
  });
}
