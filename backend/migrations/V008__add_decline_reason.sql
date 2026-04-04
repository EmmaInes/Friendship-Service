ALTER TABLE service_requests ADD COLUMN decline_reason TEXT NOT NULL DEFAULT '';
ALTER TABLE service_requests ADD COLUMN declined_by TEXT;
