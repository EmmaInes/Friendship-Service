import { isLoggedIn, getUser } from '../../api.js';

export default function home(app) {
  const user = getUser();

  app.innerHTML = `
    <section class="hero">
      <img src="/logo.svg" alt="Friendship &amp; Service" class="hero-logo" />
      <h1>Friendship &amp; Service</h1>
      <p>A community marketplace where neighbors help neighbors.</p>
      ${isLoggedIn() ? `
        <p>Welcome back, <strong>${user.display_name}</strong>!</p>
        <nav class="hero-actions">
          <a href="#/services" class="btn btn-primary">Browse Services</a>
          <a href="#/services/new" class="btn btn-secondary">Offer a Service</a>
        </nav>
      ` : `
        <nav class="hero-actions">
          <a href="#/login" class="btn btn-primary">Log In</a>
          <a href="#/register" class="btn btn-secondary">Sign Up</a>
        </nav>
      `}
    </section>
  `;
}
