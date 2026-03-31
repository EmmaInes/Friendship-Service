import { api } from '../../api.js';
import { navigate } from '../../router.js';

export default function forgotPassword(app) {
  app.innerHTML = `
    <section class="auth-page">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="auth-logo" />
      <h2>Reset Password</h2>
      <p style="text-align:center;color:var(--color-text-muted);margin-bottom:var(--space-lg);font-size:0.9rem">
        Enter your email and username to verify your identity, then choose a new password.
      </p>
      <form id="reset-form" class="auth-form">
        <label>
          Email
          <input type="email" name="email" required autocomplete="email" />
        </label>
        <label>
          Username
          <input type="text" name="username" required autocomplete="username" />
        </label>
        <label>
          New Password
          <input type="password" name="new_password" required minlength="8" autocomplete="new-password" />
        </label>
        <label>
          Confirm New Password
          <input type="password" name="confirm_password" required minlength="8" autocomplete="new-password" />
        </label>
        <p class="error-msg" id="reset-error"></p>
        <p class="success-msg" id="reset-success"></p>
        <button type="submit" class="btn btn-primary">Reset Password</button>
      </form>
      <p class="auth-switch"><a href="#/login">Back to login</a></p>
    </section>
  `;

  const form = document.getElementById('reset-form');
  const errorEl = document.getElementById('reset-error');
  const successEl = document.getElementById('reset-success');

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    errorEl.textContent = '';
    successEl.textContent = '';

    const data = Object.fromEntries(new FormData(form));

    if (data.new_password !== data.confirm_password) {
      errorEl.textContent = 'Passwords do not match';
      return;
    }

    try {
      await api.resetPassword({
        email: data.email,
        username: data.username,
        new_password: data.new_password,
      });
      successEl.textContent = 'Password updated! Redirecting to login...';
      setTimeout(() => navigate('/login'), 2000);
    } catch (err) {
      errorEl.textContent = err.error || 'Password reset failed';
    }
  });
}
