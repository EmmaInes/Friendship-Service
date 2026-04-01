-- Add work_status to service_requests for execution phase tracking
ALTER TABLE service_requests ADD COLUMN work_status TEXT NOT NULL
    DEFAULT 'not_started';

CREATE INDEX idx_requests_work_status ON service_requests(work_status);

-- Reviews table: both seeker and provider can leave one review per request
CREATE TABLE reviews (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL REFERENCES service_requests(id),
    reviewer_id TEXT NOT NULL REFERENCES users(id),
    reviewee_id TEXT NOT NULL REFERENCES users(id),
    reviewer_role TEXT NOT NULL CHECK (reviewer_role IN ('seeker', 'provider')),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(request_id, reviewer_role)
);

CREATE INDEX idx_reviews_request ON reviews(request_id);
CREATE INDEX idx_reviews_reviewee ON reviews(reviewee_id);
CREATE INDEX idx_reviews_reviewer ON reviews(reviewer_id);
