CREATE TABLE quiz_results(
    id bigserial PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    quiz_type text NOT NULL,
    params jsonb NOT NULL, -- arbitrary config from frontend
    score double precision NOT NULL, -- 0.0 .. 1.0
    question_count integer, -- optional
    duration_seconds integer, -- optional
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_quiz_results_user_created ON quiz_results(user_id, created_at DESC);
