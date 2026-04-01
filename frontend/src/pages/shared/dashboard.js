import { api, isLoggedIn, getUser } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';
import { formatPrice } from '../../utils.js';
import { renderStarInput, initStarInput } from '../../components/star-rating.js';

const WORK_STATUS_NEXT = {
  not_started: 'in_progress',
  in_progress: 'ongoing',
  ongoing: 'done',
};

const WORK_STATUS_BTN_KEY = {
  not_started: 'dashboard.btnStart',
  in_progress: 'dashboard.btnOngoing',
  ongoing: 'dashboard.btnDone',
};

function renderWorkBadge(ws) {
  const css = ws.replace('_', '-');
  return `<span class="work-badge ${css}">${t('workStatus.' + ws)}</span>`;
}

function renderActions(r) {
  const parts = [];

  // Work status advance button (provider only, when accepted)
  if (r.my_role === 'provider' && r.status === 'accepted' && WORK_STATUS_NEXT[r.work_status]) {
    const nextKey = WORK_STATUS_BTN_KEY[r.work_status];
    parts.push(`<button class="btn btn-small btn-advance" data-request-id="${r.id}" data-next="${WORK_STATUS_NEXT[r.work_status]}">${t(nextKey)}</button>`);
  }

  // Review button (both, when ongoing/done and hasn't reviewed)
  if ((r.work_status === 'ongoing' || r.work_status === 'done') && !r.has_reviewed) {
    parts.push(`<button class="btn btn-small btn-review" data-request-id="${r.id}">${t('dashboard.btnReview')}</button>`);
  } else if (r.has_reviewed) {
    parts.push(`<span class="reviewed-badge">${t('dashboard.alreadyReviewed')} ✓</span>`);
  }

  return parts.join(' ');
}

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
            : `<div class="requests-table-wrap">
                <table class="requests-table">
                <thead>
                  <tr>
                    <th>${t('dashboard.colService')}</th>
                    <th>${t('dashboard.colParty')}</th>
                    <th>${t('dashboard.colStatus')}</th>
                    <th>${t('dashboard.colWorkStatus')}</th>
                    <th>${t('dashboard.colMessage')}</th>
                    <th>${t('dashboard.colDate')}</th>
                    <th>${t('dashboard.colActions')}</th>
                  </tr>
                </thead>
                <tbody>
                  ${requests.map(r => `
                    <tr>
                      <td>${r.service_title}</td>
                      <td>${r.my_role === 'seeker' ? r.provider_name : r.seeker_name}</td>
                      <td>${t('status.' + r.status)}</td>
                      <td>${r.status === 'accepted' ? renderWorkBadge(r.work_status) : '-'}</td>
                      <td>${r.message || '-'}</td>
                      <td>${new Date(r.created_at).toLocaleDateString()}</td>
                      <td>${renderActions(r)}</td>
                    </tr>
                    <tr class="review-row" id="review-row-${r.id}" style="display:none">
                      <td colspan="7">
                        <div class="review-form" id="review-form-${r.id}">
                          <h4>${r.my_role === 'seeker' ? t('review.promptSeeker') : t('review.promptProvider')}</h4>
                          <div class="review-form-inner">
                            <label>${t('review.ratingLabel')} ${renderStarInput('rating', 0)}</label>
                            <label>${t('review.commentLabel')}
                              <textarea name="comment" rows="2" placeholder="${t('review.commentPlaceholder')}"></textarea>
                            </label>
                            <p class="error-msg" id="review-error-${r.id}"></p>
                            <p class="success-msg" id="review-success-${r.id}"></p>
                            <button class="btn btn-primary btn-submit-review" data-request-id="${r.id}">${t('review.submit')}</button>
                          </div>
                        </div>
                      </td>
                    </tr>
                  `).join('')}
                </tbody>
              </table></div>`
          }
        </section>
      </section>
    `;

    // Wire up event handlers via delegation
    app.addEventListener('click', async (e) => {
      // Advance work status
      const advanceBtn = e.target.closest('.btn-advance');
      if (advanceBtn) {
        advanceBtn.disabled = true;
        try {
          await api.updateWorkStatus(advanceBtn.dataset.requestId, advanceBtn.dataset.next);
          navigate('/dashboard'); // re-render
        } catch (err) {
          advanceBtn.disabled = false;
          alert(translateError(err.error, 'common.loading'));
        }
        return;
      }

      // Toggle review form
      const reviewBtn = e.target.closest('.btn-review');
      if (reviewBtn) {
        const row = document.getElementById(`review-row-${reviewBtn.dataset.requestId}`);
        const isVisible = row.style.display !== 'none';
        row.style.display = isVisible ? 'none' : '';
        if (!isVisible) {
          initStarInput(row);
        }
        return;
      }

      // Submit review
      const submitBtn = e.target.closest('.btn-submit-review');
      if (submitBtn) {
        const rid = submitBtn.dataset.requestId;
        const form = document.getElementById(`review-form-${rid}`);
        const rating = parseInt(form.querySelector('input[name="rating"]').value);
        const comment = form.querySelector('textarea[name="comment"]').value;
        const errorEl = document.getElementById(`review-error-${rid}`);
        const successEl = document.getElementById(`review-success-${rid}`);
        errorEl.textContent = '';
        successEl.textContent = '';

        if (!rating || rating < 1 || rating > 5) {
          errorEl.textContent = t('error.invalidRating');
          return;
        }

        submitBtn.disabled = true;
        try {
          await api.submitReview(rid, { rating, comment: comment || undefined });
          successEl.textContent = t('review.success');
          submitBtn.style.display = 'none';
        } catch (err) {
          submitBtn.disabled = false;
          errorEl.textContent = translateError(err.error, 'review.failed');
        }
      }
    });

  } catch {
    app.innerHTML = `<p class="error-msg">${t('dashboard.loadFailed')}</p>`;
  }
}
