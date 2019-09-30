CREATE TABLE sanctions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    team_id UUID NOT NULL,
    sanction_info JSONB NOT NULL,
    created_at DATE NOT NULL default CURRENT_DATE,

    CONSTRAINT team_id FOREIGN KEY (team_id) REFERENCES teams (id) ON DELETE CASCADE,
    CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
)