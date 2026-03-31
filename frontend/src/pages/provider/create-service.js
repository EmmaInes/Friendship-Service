import { api, isLoggedIn } from '../../api.js';
import { navigate } from '../../router.js';

const CATEGORIES = [
  'Tutoring', 'Home Repair', 'Gardening', 'Pet Care', 'Cleaning',
  'Transportation', 'Technology', 'Cooking', 'Childcare', 'Other'
];

export default function createService(app) {
  if (!isLoggedIn()) {
    navigate('/login');
    return;
  }

  app.innerHTML = `
    <section>
      <h2>Offer a Service</h2>
      <form id="service-form" class="service-form">
        <label>
          Title
          <input type="text" name="title" required maxlength="200" placeholder="e.g. Math Tutoring for High School" />
        </label>
        <label>
          Category
          <select name="category" required>
            <option value="">Select a category</option>
            ${CATEGORIES.map(c => `<option value="${c}">${c}</option>`).join('')}
          </select>
        </label>
        <label>
          Description
          <textarea name="description" required placeholder="Describe what you offer, your experience, availability..."></textarea>
        </label>
        <label>
          Pricing
          <select name="price_type">
            <option value="negotiable">Negotiable</option>
            <option value="free">Free</option>
            <option value="fixed">Fixed Price</option>
            <option value="hourly">Hourly Rate</option>
          </select>
        </label>
        <label id="price-label" style="display:none">
          Amount (in dollars)
          <input type="number" name="price" min="0" step="0.01" placeholder="0.00" />
        </label>
        <label>
          Location (optional)
          <input type="text" name="location" placeholder="e.g. Downtown, Online, etc." />
        </label>
        <p class="error-msg" id="service-error"></p>
        <button type="submit" class="btn btn-primary">Create Service</button>
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
        errorEl.textContent = 'Please enter a valid price';
        return;
      }
      data.price_cents = Math.round(dollars * 100);
    }

    try {
      await api.createService(data);
      navigate('/dashboard');
    } catch (err) {
      errorEl.textContent = err.error || 'Failed to create service';
    }
  });
}
