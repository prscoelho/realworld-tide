CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL,
    image TEXT,
    bio TEXT
);