import { api, isLoggedIn } from '../../api.js';
import { navigate } from '../../router.js';
import { t } from '../../i18n/i18n.js';
import { CATEGORIES } from '../../utils.js';

const AVAILABILITY_KEYS = ['weekdays', 'weekends', 'evenings', 'flexible'];
const EXPERIENCE_KEYS = ['beginner', 'intermediate', 'expert'];

export default async function providerSurvey(app) {
  if (!isLoggedIn()) { navigate('/login'); return; }

  app.innerHTML = `<p>${t('common.loading')}</p>`;

  let existing = null;
  try {
    const surveys = await api.getMySurveys();
    existing = surveys.find(s => s.survey_type === 'provider');
  } catch { /* first time */ }

  const selectedCats = existing?.categories || [];

  app.innerHTML = `
    <section class="survey-page">
      <h2>${t('providerSurvey.title')}</h2>
      <p class="survey-intro">${t('providerSurvey.intro')}</p>
      <form id="provider-survey-form" class="service-form">
        <fieldset>
          <legend>${t('providerSurvey.categoriesLegend')}</legend>
          <div class="checkbox-grid">
            ${CATEGORIES.map(c => `
              <label class="checkbox-label">
                <input type="checkbox" name="categories" value="${c}" ${selectedCats.includes(c) ? 'checked' : ''} />
                ${t('category.' + c)}
              </label>
            `).join('')}
          </div>
        </fieldset>

        <label>
          ${t('providerSurvey.experience')}
          <select name="experience_level">
            ${EXPERIENCE_KEYS.map(e => `<option value="${e}" ${existing?.experience_level === e ? 'selected' : ''}>${t('experience.' + e)}</option>`).join('')}
          </select>
        </label>

        <label>
          ${t('providerSurvey.availability')}
          <select name="availability">
            ${AVAILABILITY_KEYS.map(a => `<option value="${a}" ${existing?.availability === a ? 'selected' : ''}>${t('availability.' + a)}</option>`).join('')}
          </select>
        </label>

        <label>
          ${t('providerSurvey.location')}
          <input type="text" name="location_preference" placeholder="${t('providerSurvey.locationPlaceholder')}" value="${existing?.location_preference || ''}" />
        </label>

        <label>
          ${t('providerSurvey.skills')}
          <textarea name="description" rows="4" placeholder="${t('providerSurvey.skillsPlaceholder')}">${existing?.description || ''}</textarea>
        </label>

        <p class="error-msg" id="survey-error"></p>
        <p class="success-msg" id="survey-success"></p>
        <button type="submit" class="btn btn-primary">${existing ? t('providerSurvey.update') : t('providerSurvey.submit')}</button>
      </form>
    </section>
  `;

  const form = document.getElementById('provider-survey-form');
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const errorEl = document.getElementById('survey-error');
    const successEl = document.getElementById('survey-success');
    errorEl.textContent = '';
    successEl.textContent = '';

    const checked = [...form.querySelectorAll('input[name="categories"]:checked')].map(el => el.value);
    if (checked.length === 0) {
      errorEl.textContent = t('providerSurvey.selectOne');
      return;
    }

    try {
      await api.saveSurvey({
        survey_type: 'provider',
        categories: checked,
        availability: form.availability.value,
        location_preference: form.location_preference.value || undefined,
        experience_level: form.experience_level.value,
        description: form.description.value || undefined,
      });
      successEl.textContent = t('providerSurvey.saved');
    } catch (err) {
      errorEl.textContent = err.error || t('providerSurvey.failed');
    }
  });
}
