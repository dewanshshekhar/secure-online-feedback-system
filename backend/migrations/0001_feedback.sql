BEGIN;
CREATE TABLE IF NOT EXISTS feedback (
    id UUID PRIMARY KEY,
    category TEXT NOT NULL CHECK (category IN ('teaching', 'course-content', 'facilities', 'other')),
    message TEXT NOT NULL CHECK (char_length(btrim(message)) BETWEEN 1 AND 2000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS feedback_review_order ON feedback (created_at DESC, id DESC);
COMMENT ON TABLE feedback IS 'Submitted feedback only. No seed records or submitter identity.';
COMMIT;
