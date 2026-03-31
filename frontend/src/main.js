const app = document.getElementById('app');

app.innerHTML = `
  <header>
    <h1>Friendship&amp;Service</h1>
    <p>Connecting service providers with those who need them.</p>
  </header>
  <main>
    <p>Frontend is running.</p>
  </main>
`;

// Test API connectivity
fetch('/api/health')
  .then(res => res.json())
  .then(data => {
    const main = document.querySelector('main');
    const status = document.createElement('p');
    status.textContent = `API status: ${data.status}`;
    main.appendChild(status);
  })
  .catch(() => {
    const main = document.querySelector('main');
    const status = document.createElement('p');
    status.textContent = 'API: not connected (start the backend)';
    main.appendChild(status);
  });
