CREATE TABLE users (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL,
    firstname VARCHAR NOT NULL,
    lastname VARCHAR NOT NULL,
    nickname VARCHAR,
    login VARCHAR UNIQUE NOT NULL,
    password VARCHAR UNIQUE NOT NULL,
    email VARCHAR UNIQUE,

    FOREIGN KEY (team_id) REFERENCES teams (id) ON DELETE CASCADE
)