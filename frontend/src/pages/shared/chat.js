import { api, isLoggedIn, getUser } from '../../api.js';
import { navigate } from '../../router.js';
import { t, translateError } from '../../i18n/i18n.js';

export default async function chat(app, requestId) {
  if (!isLoggedIn()) { navigate('/login'); return; }

  const user = getUser();
  app.innerHTML = `<p>${t('common.loading')}</p>`;

  // Load request info to show context
  let requests;
  try {
    requests = await api.getMyRequests();
  } catch {
    app.innerHTML = `<p class="error-msg">${t('chat.loadFailed')}</p>`;
    return;
  }

  const req = requests.find(r => r.id === requestId);
  if (!req) {
    app.innerHTML = `<p class="error-msg">${t('chat.loadFailed')}</p>`;
    return;
  }

  const otherName = req.my_role === 'seeker' ? req.provider_name : req.seeker_name;

  app.innerHTML = `
    <div class="chat-page">
      <div class="chat-header">
        <div class="chat-header-top">
          <a href="#/dashboard" class="chat-back">\u2190 ${t('nav.dashboard')}</a>
          ${req.my_role === 'seeker' && req.status === 'accepted' && req.work_status === 'not_started'
            ? `<button id="chat-accept-offer" class="btn btn-primary btn-small">${t('dashboard.btnAcceptOffer')}</button>`
            : ''}
        </div>
        <h3>${t('chat.with', { name: otherName })}</h3>
        <p class="chat-about">${t('chat.about', { serviceTitle: req.service_title })}</p>
      </div>
      <div class="chat-messages" id="chat-messages"></div>
      <form class="chat-input" id="chat-input-form">
        <input type="text" id="chat-input" placeholder="${t('chat.placeholder')}" maxlength="2000" autocomplete="off" />
        <button type="submit" class="btn btn-primary">${t('chat.send')}</button>
      </form>
    </div>
  `;

  const messagesEl = document.getElementById('chat-messages');
  const inputForm = document.getElementById('chat-input-form');
  const inputEl = document.getElementById('chat-input');
  let lastTimestamp = null;

  function renderMessages(messages) {
    if (messages.length === 0 && !lastTimestamp) {
      messagesEl.innerHTML = `<p class="chat-empty">${t('chat.empty')}</p>`;
      return;
    }

    // Remove empty state if it's there
    const emptyEl = messagesEl.querySelector('.chat-empty');
    if (emptyEl) emptyEl.remove();

    for (const msg of messages) {
      const isMine = msg.sender_id === user.id;
      const time = new Date(msg.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

      const bubble = document.createElement('div');
      bubble.className = `msg ${isMine ? 'msg-mine' : 'msg-theirs'}`;
      bubble.innerHTML = `
        <div class="msg-body">${escapeHtml(msg.body)}</div>
        <div class="msg-time">${isMine ? '' : msg.sender_name + ' \u00B7 '}${time}</div>
      `;
      messagesEl.appendChild(bubble);

      lastTimestamp = msg.created_at;
    }

    messagesEl.scrollTop = messagesEl.scrollHeight;
  }

  function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  // Accept Offer button
  const acceptBtn = document.getElementById('chat-accept-offer');
  if (acceptBtn) {
    acceptBtn.addEventListener('click', async () => {
      acceptBtn.disabled = true;
      try {
        await api.updateWorkStatus(requestId, 'in_progress');
        navigate('/dashboard');
      } catch (err) {
        acceptBtn.disabled = false;
        acceptBtn.textContent = translateError(err.error, 'chat.loadFailed');
      }
    });
  }

  // Initial load
  try {
    const messages = await api.getMessages(requestId);
    renderMessages(messages);
  } catch {
    messagesEl.innerHTML = `<p class="error-msg">${t('chat.loadFailed')}</p>`;
  }

  // Poll for new messages
  const pollInterval = setInterval(async () => {
    try {
      const messages = await api.getMessages(requestId, lastTimestamp);
      if (messages.length > 0) {
        renderMessages(messages);
      }
    } catch { /* silent retry next interval */ }
  }, 5000);

  // Send message
  inputForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const text = inputEl.value.trim();
    if (!text) return;

    inputEl.value = '';
    try {
      await api.sendMessage(requestId, text);
      // Immediately fetch to show our own message
      const messages = await api.getMessages(requestId, lastTimestamp);
      if (messages.length > 0) {
        renderMessages(messages);
      }
    } catch {
      inputEl.value = text; // restore on failure
    }
  });

  inputEl.focus();

  // Cleanup function returned to router
  return () => {
    clearInterval(pollInterval);
  };
}
