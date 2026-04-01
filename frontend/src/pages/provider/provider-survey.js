import { api, isLoggedIn } from '../../api.js';
import { navigate } from '../../router.js';

const CATEGORIES = [
  'Tutoring', 'Home Repair', 'Gardening', 'Pet Care', 'Cleaning',
  'Transportation', 'Technology', 'Cooking', 'Childcare', 'Other'
];

const AVAILABILITY = ['Weekdays', 'Weekends', 'Evenings', 'Flexible'];
const EXPERIENCE = ['Beginner', 'Intermediate', 'Expert'];

export default async function providerSurvey(app) {
  if (!isLoggedIn()) { navigate('/login'); return; }

  app.innerHTML = '<p>Loading...</p>';

  let existing = null;
  try {
    const surveys = await api.getMySurveys();
    existing = surveys.find(s => s.survey_type === 'provider');
  } catch { /* first time */ }

  const selectedCats = existing?.categories || [];

  app.innerHTML = `
    <section class="survey-page">
      <h2>Provider Survey</h2>
      <p class="survey-intro">Tell us about the services you can offer so we can connect you with people who need help.</p>
      <form id="provider-survey-form" class="service-form">
        <fieldset>
          <legend>What categories can you help with?</legend>
          <div class="checkbox-grid">
            ${CATEGORIES.map(c => `
              <label class="checkbox-label">
                <input type="checkbox" name="categories" value="${c}" ${selectedCats.includes(c) ? 'checked' : ''} />
                ${c}
              </label>
            `).join('')}
          </div>
        </fieldset>

        <label>
          Experience Level
          <select name="experience_level">
            ${EXPERIENCE.map(e => `<option value="${e.toLowerCase()}" ${existing?.experience_level === e.toLowerCase() ? 'selected' : ''}>${e}</option>`).join('')}
          </select>
        </label>

        <label>
          Availability
          <select name="availability">
            ${AVAILABILITY.map(a => `<option value="${a.toLowerCase()}" ${existing?.availability === a.toLowerCase() ? 'selected' : ''}>${a}</option>`).join('')}
          </select>
        </label>

        <label>
          Preferred Location / Area
          <input type="text" name="location_preference" placeholder="e.g. Downtown, Online, Citywide" value="${existing?.location_preference || ''}" />
        </label>

        <label>
          Tell us more about your skills and experience
          <textarea name="description" rows="4" placeholder="What makes you great at these services?">${existing?.description || ''}</textarea>
        </label>

        <p class="error-msg" id="survey-error"></p>
        <p class="success-msg" id="survey-success"></p>
        <button type="submit" class="btn btn-primary">${existing ? 'Update Survey' : 'Submit Survey'}</button>
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
      errorEl.textContent = 'Please select at least one category';
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
      successEl.textContent = 'Survey saved!';
    } catch (err) {
      errorEl.textContent = err.error || 'Failed to save survey';
    }
  });
}
