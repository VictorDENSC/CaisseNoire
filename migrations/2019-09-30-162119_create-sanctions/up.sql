CREATE TABLE sanctions (
    id UUID PRIMARY KEY,
    user_id UUID,
    team_id UUID,
    associated_rule JSONB NOT NULL,
    created_at DATE NOT NULL,

    CONSTRAINT team_id FOREIGN KEY (team_id) REFERENCES teams (id) ON DELETE CASCADE,
    CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
)