CREATE TABLE messages (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL REFERENCES service_requests(id),
    sender_id TEXT NOT NULL REFERENCES users(id),
    body TEXT NOT NULL,
    read_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_messages_request ON messages(request_id, created_at);
CREATE INDEX idx_messages_unread ON messages(request_id, sender_id, read_at);
