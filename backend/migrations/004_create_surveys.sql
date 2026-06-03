CREATE TABLE surveys (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    survey_type TEXT NOT NULL CHECK (survey_type IN ('provider', 'seeker')),
    categories TEXT NOT NULL DEFAULT '[]',
    budget_min BIGINT,
    budget_max BIGINT,
    availability TEXT NOT NULL DEFAULT '',
    location_preference TEXT NOT NULL DEFAULT '',
    experience_level TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    urgency TEXT NOT NULL DEFAULT 'flexible',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, survey_type)
);

CREATE INDEX idx_surveys_user ON surveys(user_id);
CREATE INDEX idx_surveys_type ON surveys(survey_type);
