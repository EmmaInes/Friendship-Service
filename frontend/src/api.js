const TOKEN_KEY = 'fs_token';
const USER_KEY = 'fs_user';

export function getToken() {
  return localStorage.getItem(TOKEN_KEY);
}

export function getUser() {
  const raw = localStorage.getItem(USER_KEY);
  return raw ? JSON.parse(raw) : null;
}

export function setAuth(token, user) {
  localStorage.setItem(TOKEN_KEY, token);
  localStorage.setItem(USER_KEY, JSON.stringify(user));
}

export function clearAuth() {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
}

export function isLoggedIn() {
  return !!getToken();
}

async function request(method, path, body) {
  const headers = { 'Content-Type': 'application/json' };
  const token = getToken();
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`/api${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });

  const data = await res.json();
  if (!res.ok) throw { status: res.status, ...data };
  return data;
}

export const api = {
  register: (data) => request('POST', '/auth/register', data),
  login: (data) => request('POST', '/auth/login', data),
  me: () => request('GET', '/auth/me'),
  resetPassword: (data) => request('POST', '/auth/reset-password', data),
  googleLogin: (data) => request('POST', '/auth/google', data),
  getServices: () => request('GET', '/services'),
  createService: (data) => request('POST', '/services', data),
  getService: (id) => request('GET', `/services/${id}`),
  requestService: (id, data) => request('POST', `/services/${id}/request`, data),
  getMyServices: () => request('GET', '/services/mine'),
  getMyRequests: () => request('GET', '/requests/mine'),
  updateRequestStatus: (id, status, reason) => request('PATCH', `/requests/${id}`, { status, reason }),
  updateWorkStatus: (id, workStatus) => request('PATCH', `/requests/${id}/work-status`, { work_status: workStatus }),
  submitReview: (requestId, data) => request('POST', `/requests/${requestId}/review`, data),
  getRequestReviews: (requestId) => request('GET', `/requests/${requestId}/reviews`),
  getUserRatings: (userId) => request('GET', `/users/${userId}/ratings`),
  getMessages: (requestId, after) => request('GET', `/requests/${requestId}/messages${after ? '?after=' + encodeURIComponent(after) : ''}`),
  sendMessage: (requestId, body) => request('POST', `/requests/${requestId}/messages`, { body }),
  getUnreadCount: () => request('GET', '/messages/unread-count'),
  saveSurvey: (data) => request('POST', '/surveys', data),
  getMySurveys: () => request('GET', '/surveys/mine'),
  getSuggestions: () => request('GET', '/suggestions'),
};
