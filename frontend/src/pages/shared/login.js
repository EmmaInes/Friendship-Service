import { api, setAuth } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';
import { GOOGLE_CLIENT_ID } from '../../config.js';

export default function login(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>${t('login.title')}</h2>
      <p style="text-align:center;color:var(--color-text-muted);margin-bottom:var(--space-lg)">${t('login.googlePrompt')}</p>
      <div id="g_id_signin" class="google-btn-wrap"></div>
      <p class="error-msg" id="login-error" style="text-align:center;margin-top:var(--space-md)"></p>
    </section>
  `;

  const errorEl = document.getElementById('login-error');

  async function handleCredentialResponse(response) {
    errorEl.textContent = '';
    try {
      const res = await api.googleLogin({ credential: response.credential });
      setAuth(res.token, res.user);
      navigate('/');
    } catch (err) {
      errorEl.textContent = translateError(err.error, 'login.googleFailed');
    }
  }

  // Wait for GSI library to load
  function initGSI() {
    if (typeof google === 'undefined' || !google.accounts) {
      setTimeout(initGSI, 200);
      return;
    }
    google.accounts.id.initialize({
      client_id: GOOGLE_CLIENT_ID,
      callback: handleCredentialResponse,
    });
    google.accounts.id.renderButton(
      document.getElementById('g_id_signin'),
      { theme: 'outline', size: 'large', text: 'signin_with', width: 320 }
    );
  }

  initGSI();
}
