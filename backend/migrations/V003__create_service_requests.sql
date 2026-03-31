CREATE TABLE service_requests (
    id TEXT PRIMARY KEY NOT NULL,
    service_id TEXT NOT NULL REFERENCES services(id),
    seeker_id TEXT NOT NULL REFERENCES users(id),
    message TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL CHECK (status IN ('pending', 'accepted', 'declined', 'completed', 'cancelled')) DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_requests_service ON service_requests(service_id);
CREATE INDEX idx_requests_seeker ON service_requests(seeker_id);
CREATE INDEX idx_requests_status ON service_requests(status);
