import { api, setAuth } from '../../api.js';
import { navigate } from '../../router.js';

export default function register(app) {
  app.innerHTML = `
    <section class="auth-page">
      <h2>Create Account</h2>
      <form id="register-form" class="auth-form">
        <label>
          Display Name
          <input type="text" name="display_name" required minlength="1" maxlength="100" />
        </label>
        <label>
          Username
          <input type="text" name="username" required minlength="3" maxlength="30" autocomplete="username" />
        </label>
        <label>
          Email
          <input type="email" name="email" required autocomplete="email" />
        </label>
        <label>
          Password
          <input type="password" name="password" required minlength="8" autocomplete="new-password" />
        </label>
        <p class="error-msg" id="register-error"></p>
        <button type="submit" class="btn btn-primary">Sign Up</button>
      </form>
      <p class="auth-switch">Already have an account? <a href="#/login">Log in</a></p>
    </section>
  `;

  const form = document.getElementById('register-form');
  const errorEl = document.getElementById('register-error');

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.textContent = '';

    const data = Object.fromEntries(new FormData(form));

    try {
      const res = await api.register(data);
      setAuth(res.token, res.user);
      navigate('/');
    } catch (err) {
      errorEl.textContent = err.error || 'Registration failed';
    }
  });
}
