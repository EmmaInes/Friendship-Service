import { t } from '../i18n/i18n.js';

export function renderStars(rating, count) {
  if (count === 0 || !rating) {
    return `<span class="rating-none">${t('rating.noReviews')}</span>`;
  }
  const full = Math.round(rating);
  const stars = '★'.repeat(full) + '☆'.repeat(5 - full);
  return `<span class="stars">${stars}</span> <span class="review-count">${rating} ${t('rating.count', { count })}</span>`;
}

export function renderStarInput(name, currentValue = 0) {
  return `
    <div class="star-input-group" data-name="${name}">
      ${[1, 2, 3, 4, 5].map(i => `
        <span class="star-input ${i <= currentValue ? 'active' : ''}" data-value="${i}">★</span>
      `).join('')}
      <input type="hidden" name="${name}" value="${currentValue}" />
    </div>
  `;
}

export function initStarInput(container) {
  const group = container.querySelector('.star-input-group');
  if (!group) return;

  group.addEventListener('click', (e) => {
    const star = e.target.closest('.star-input');
    if (!star) return;

    const value = parseInt(star.dataset.value);
    const hidden = group.querySelector('input[type="hidden"]');
    hidden.value = value;

    group.querySelectorAll('.star-input').forEach((s, i) => {
      s.classList.toggle('active', i < value);
    });
  });
}
