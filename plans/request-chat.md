# Plan: Request Chat (Seeker–Provider Messaging)

**Status:** Planned  
**Created:** 2026-04-01  
**Depends on:** Satisfaction Surveys & Ratings (work_status flow)

---

## Overview

Add a messaging system scoped to service requests so seekers and providers can communicate details — scheduling, scope, pricing, clarifications — without leaving the platform. Chat is tied to a `service_request`, not general user-to-user, keeping it focused and manageable.

---

## Architecture: Polling-Based Chat

To stay consistent with the project's lean, zero-dependency philosophy, the chat uses **polling** rather than WebSockets:

- Messages stored in SQLite
- Frontend polls for new messages every 5 seconds while the chat is open
- Unread message count shown as a badge on the Dashboard nav link
- Simple and robust — no WebSocket server, no connection management

---

## Database Changes

### Migration V006: `V006__create_messages.sql`

```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL REFERENCES service_requests(id),
    sender_id TEXT NOT NULL REFERENCES users(id),
    body TEXT NOT NULL,
    read_at TEXT,  -- NULL = unread, timestamp = read
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_messages_request ON messages(request_id, created_at);
CREATE INDEX idx_messages_unread ON messages(request_id, sender_id, read_at);
```

Key decisions:
- `read_at` is per-message — when the other party opens the chat, all messages from the sender get marked as read
- No `recipient_id` needed — it's implied (the other party in the request)
- Messages ordered by `created_at` ascending

---

## Backend Endpoints

### 1. GET `/api/requests/{id}/messages` — Fetch messages
- Auth required; only seeker or provider of this request
- Query param: `?after={timestamp}` for incremental polling (only fetch new messages)
- Returns messages array ordered by created_at ASC
- Side effect: marks all messages from the OTHER party as read

### 2. POST `/api/requests/{id}/messages` — Send a message
- Auth required; only seeker or provider of this request
- Validate: request status must be "accepted" (or pending — allow pre-acceptance discussion)
- Body: `{ "body": "message text" }`
- Max length: 2000 characters

### 3. GET `/api/messages/unread-count` — Unread message count
- Auth required
- Returns: `{ "count": 5 }` — total unread messages across all requests for current user
- Used by nav badge

---

## Frontend Changes

### API client (`api.js`)

```js
getMessages: (requestId, after) => request('GET', `/requests/${requestId}/messages${after ? '?after=' + after : ''}`),
sendMessage: (requestId, body) => request('POST', `/requests/${requestId}/messages`, { body }),
getUnreadCount: () => request('GET', '/messages/unread-count'),
```

### Chat page (`pages/shared/chat.js`)

New route: `#/chat/{requestId}`

Layout:
- Header: service title + other party's name + back to dashboard link
- Message list: scrollable container, messages styled as bubbles (left = other, right = mine)
- Input bar: text input + send button at the bottom
- Auto-scroll to bottom on new messages

Polling:
- On mount, fetch all messages
- Set `setInterval` every 5 seconds to fetch new messages (using `?after=` timestamp of last message)
- Cleanup interval on page leave (router cleanup function)

### Nav badge (`main.js`)

- Poll unread count every 30 seconds when logged in
- Display badge on Dashboard nav link: `Dashboard (3)` or a dot indicator
- Clear polling on logout

### Dashboard integration (`pages/shared/dashboard.js`)

- Add a "Chat" button on each accepted request row
- Show unread count per request if available
- Link to `#/chat/{requestId}`

### Route registration

```js
route('/chat/:id', chat);
```

---

## i18n Keys

```
chat.title: "Chat"
chat.placeholder: "Type a message..."
chat.send: "Send"
chat.empty: "No messages yet. Start the conversation!"
chat.loadFailed: "Failed to load messages"
chat.sendFailed: "Failed to send message"
chat.with: "Chat with {name}"
chat.about: "About: {serviceTitle}"

dashboard.btnChat: "Chat"
dashboard.unread: "{count} new"

nav.unread: "{count}"
```

---

## CSS Additions

```css
/* Chat layout */
.chat-page { display: flex; flex-direction: column; height: calc(100vh - 80px); max-width: 700px; margin: 0 auto; }
.chat-header { padding: var(--space-md); border-bottom: 1px solid var(--color-border); }
.chat-messages { flex: 1; overflow-y: auto; padding: var(--space-md); display: flex; flex-direction: column; gap: var(--space-sm); }
.chat-input { display: flex; gap: var(--space-sm); padding: var(--space-md); border-top: 1px solid var(--color-border); }
.chat-input input { flex: 1; }

/* Message bubbles */
.msg { max-width: 75%; padding: var(--space-sm) var(--space-md); border-radius: 12px; font-size: 0.9rem; }
.msg-mine { align-self: flex-end; background: var(--color-primary); color: #fff; border-bottom-right-radius: 4px; }
.msg-theirs { align-self: flex-start; background: var(--color-border); border-bottom-left-radius: 4px; }
.msg-time { font-size: 0.7rem; opacity: 0.6; margin-top: 2px; }

/* Unread badge */
.nav-badge { background: var(--color-error); color: #fff; font-size: 0.65rem; padding: 1px 5px; border-radius: 10px; margin-left: 4px; }
```

---

## Implementation Phases

### Phase 1: Database & Backend
1. Create migration V006
2. Create `handlers/messages.rs` with 3 endpoints
3. Register routes in `main.rs`

### Phase 2: Chat UI
1. Create `pages/shared/chat.js` with message list, input, polling
2. Add route in `main.js`
3. Add API methods to `api.js`
4. Add chat CSS

### Phase 3: Dashboard & Nav Integration
1. Add "Chat" button to dashboard request rows
2. Add unread count polling to `main.js`
3. Show nav badge with unread count

### Phase 4: i18n & Polish
1. Add all keys to en/es/pt locale files
2. Handle edge cases: empty chat, long messages, rapid polling
3. Test full flow: accept request → chat → advance status → review

---

## Considerations

- **Polling frequency**: 5s for active chat, 30s for nav badge. Adjust if needed.
- **Message ordering**: SQLite `strftime` has second precision. If two messages arrive in the same second, `id` (UUID) provides stable ordering as tiebreaker.
- **Future upgrade path**: If real-time becomes important, the same API contract works with WebSocket push — just add a notification channel alongside the existing polling endpoints.
- **Moderation**: For now, no content filtering. Could add basic profanity filter or report button in a future phase.
- **Mobile responsiveness**: Chat layout should work on small screens — the flex column layout handles this naturally.
