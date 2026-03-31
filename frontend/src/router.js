const routes = {};
const paramRoutes = []; // { pattern: RegExp, paramNames: string[], handler: fn }
let currentCleanup = null;

export function route(path, handler) {
  // Check if path has parameters like :id
  if (path.includes(':')) {
    const paramNames = [];
    const pattern = path.replace(/:(\w+)/g, (_, name) => {
      paramNames.push(name);
      return '([^/]+)';
    });
    paramRoutes.push({ pattern: new RegExp(`^${pattern}$`), paramNames, handler });
  } else {
    routes[path] = handler;
  }
}

export function navigate(path) {
  window.location.hash = path;
}

export function start(appEl) {
  async function render() {
    const hash = window.location.hash.slice(1) || '/';

    if (currentCleanup) {
      currentCleanup();
      currentCleanup = null;
    }

    // Try exact match first
    if (routes[hash]) {
      appEl.innerHTML = '';
      currentCleanup = await routes[hash](appEl) || null;
      return;
    }

    // Try parameterized routes
    for (const { pattern, paramNames, handler } of paramRoutes) {
      const match = hash.match(pattern);
      if (match) {
        const params = {};
        paramNames.forEach((name, i) => (params[name] = match[i + 1]));
        appEl.innerHTML = '';
        currentCleanup = await handler(appEl, ...Object.values(params)) || null;
        return;
      }
    }

    // 404
    if (routes['/404']) {
      appEl.innerHTML = '';
      currentCleanup = await routes['/404'](appEl) || null;
    }
  }

  window.addEventListener('hashchange', render);
  render();
}
