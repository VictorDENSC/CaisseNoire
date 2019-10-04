CREATE TABLE teams (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    admin_password VARCHAR NOT NULL,
    rules JSONB[] NOT NULL,
    
    CONSTRAINT name UNIQUE (name)
)