CREATE TABLE users (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL,
    firstname VARCHAR NOT NULL,
    lastname VARCHAR NOT NULL,
    nickname VARCHAR,
    login VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    email VARCHAR,

    CONSTRAINT team_id FOREIGN KEY (team_id) REFERENCES teams (id) ON DELETE CASCADE,
    CONSTRAINT login UNIQUE (login),
    CONSTRAINT email UNIQUE (email)
)