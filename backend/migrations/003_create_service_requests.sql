CREATE TABLE service_requests (
    id TEXT PRIMARY KEY NOT NULL,
    service_id TEXT NOT NULL REFERENCES services(id),
    seeker_id TEXT NOT NULL REFERENCES users(id),
    message TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'declined', 'completed', 'cancelled')),
    work_status TEXT NOT NULL DEFAULT 'not_started',
    decline_reason TEXT NOT NULL DEFAULT '',
    declined_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_requests_service ON service_requests(service_id);
CREATE INDEX idx_requests_seeker ON service_requests(seeker_id);
CREATE INDEX idx_requests_status ON service_requests(status);
CREATE INDEX idx_requests_work_status ON service_requests(work_status);
