CREATE TABLE surveys (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    survey_type TEXT NOT NULL CHECK (survey_type IN ('provider', 'seeker')),
    categories TEXT NOT NULL DEFAULT '[]',       -- JSON array of category strings
    budget_min INTEGER,                          -- seeker: min budget in cents
    budget_max INTEGER,                          -- seeker: max budget in cents
    availability TEXT NOT NULL DEFAULT '',        -- e.g. "weekdays", "weekends", "evenings", "flexible"
    location_preference TEXT NOT NULL DEFAULT '', -- preferred area
    experience_level TEXT NOT NULL DEFAULT '',    -- provider: "beginner", "intermediate", "expert"
    description TEXT NOT NULL DEFAULT '',         -- free-text about needs or skills
    urgency TEXT NOT NULL DEFAULT 'flexible',     -- seeker: "urgent", "this_week", "this_month", "flexible"
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(user_id, survey_type)
);

CREATE INDEX idx_surveys_user ON surveys(user_id);
CREATE INDEX idx_surveys_type ON surveys(survey_type);
