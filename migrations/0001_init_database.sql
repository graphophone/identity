CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(24) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_profile (
    user_id BIGSERIAL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    email TEXT UNIQUE,
    first_name TEXT,
    last_name TEXT,
    bio TEXT,
    city TEXT,
    country TEXT,
    avatar_url TEXT
);