export default function notFound(app) {
  app.innerHTML = `
    <section class="not-found">
      <h2>404</h2>
      <p>Page not found.</p>
      <a href="#/" class="btn btn-primary">Go Home</a>
    </section>
  `;
}
