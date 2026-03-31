import { api, setAuth } from '../../api.js';
import { navigate } from '../../router.js';

export default function login(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>Log In</h2>
      <form id="login-form" class="auth-form">
        <label>
          Email
          <input type="email" name="email" required autocomplete="email" />
        </label>
        <label>
          Password
          <input type="password" name="password" required autocomplete="current-password" />
        </label>
        <p class="error-msg" id="login-error"></p>
        <button type="submit" class="btn btn-primary">Log In</button>
      </form>
      <p class="auth-switch">Don't have an account? <a href="#/register">Sign up</a></p>
    </section>
  `;

  const form = document.getElementById('login-form');
  const errorEl = document.getElementById('login-error');

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.textContent = '';

    const data = Object.fromEntries(new FormData(form));

    try {
      const res = await api.login(data);
      setAuth(res.token, res.user);
      navigate('/');
    } catch (err) {
      errorEl.textContent = err.error || 'Login failed';
    }
  });
}
