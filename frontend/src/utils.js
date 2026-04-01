import { t } from './i18n/i18n.js';

export const CATEGORIES = [
  'Tutoring', 'Home Repair', 'Gardening', 'Pet Care', 'Cleaning',
  'Transportation', 'Technology', 'Cooking', 'Childcare', 'Other'
];

export function formatPrice(cents, type) {
  if (type === 'free') return t('price.free');
  if (type === 'negotiable') return t('price.negotiable');
  if (cents == null) return type;
  const dollars = (cents / 100).toFixed(2);
  return `$${dollars}${type === 'hourly' ? t('price.perHour') : ''}`;
}
