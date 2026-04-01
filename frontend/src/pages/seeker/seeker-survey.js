import { api, isLoggedIn } from '../../api.js';
import { navigate } from '../../router.js';

const CATEGORIES = [
  'Tutoring', 'Home Repair', 'Gardening', 'Pet Care', 'Cleaning',
  'Transportation', 'Technology', 'Cooking', 'Childcare', 'Other'
];

const AVAILABILITY = ['Weekdays', 'Weekends', 'Evenings', 'Flexible'];
const URGENCY = [
  { value: 'urgent', label: 'Urgent (ASAP)' },
  { value: 'this_week', label: 'This week' },
  { value: 'this_month', label: 'This month' },
  { value: 'flexible', label: 'Flexible / No rush' },
];

export default async function seekerSurvey(app) {
  if (!isLoggedIn()) { navigate('/login'); return; }

  app.innerHTML = '<p>Loading...</p>';

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
      <h2>Seeker Survey</h2>
      <p class="survey-intro">Tell us what services you're looking for and we'll suggest the best matches.</p>
      <form id="seeker-survey-form" class="service-form">
        <fieldset>
          <legend>What kind of help do you need?</legend>
          <div class="checkbox-grid">
            ${CATEGORIES.map(c => `
              <label class="checkbox-label">
                <input type="checkbox" name="categories" value="${c}" ${selectedCats.includes(c) ? 'checked' : ''} />
                ${c}
              </label>
            `).join('')}
          </div>
        </fieldset>

        <fieldset>
          <legend>Budget Range (optional)</legend>
          <div class="inline-fields">
            <label>
              Min ($)
              <input type="number" name="budget_min" min="0" step="0.01" placeholder="0.00" value="${budgetMin}" />
            </label>
            <label>
              Max ($)
              <input type="number" name="budget_max" min="0" step="0.01" placeholder="100.00" value="${budgetMax}" />
            </label>
          </div>
        </fieldset>

        <label>
          How urgent is your need?
          <select name="urgency">
            ${URGENCY.map(u => `<option value="${u.value}" ${existing?.urgency === u.value ? 'selected' : ''}>${u.label}</option>`).join('')}
          </select>
        </label>

        <label>
          Preferred Availability
          <select name="availability">
            ${AVAILABILITY.map(a => `<option value="${a.toLowerCase()}" ${existing?.availability === a.toLowerCase() ? 'selected' : ''}>${a}</option>`).join('')}
          </select>
        </label>

        <label>
          Preferred Location / Area
          <input type="text" name="location_preference" placeholder="e.g. Downtown, Online" value="${existing?.location_preference || ''}" />
        </label>

        <label>
          Describe what you're looking for
          <textarea name="description" rows="4" placeholder="Any specific requirements or preferences?">${existing?.description || ''}</textarea>
        </label>

        <p class="error-msg" id="survey-error"></p>
        <p class="success-msg" id="survey-success"></p>
        <button type="submit" class="btn btn-primary">${existing ? 'Update Survey' : 'Submit Survey'}</button>
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
      errorEl.textContent = 'Please select at least one category';
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
      successEl.textContent = 'Survey saved! Check your suggestions.';
    } catch (err) {
      errorEl.textContent = err.error || 'Failed to save survey';
    }
  });
}
