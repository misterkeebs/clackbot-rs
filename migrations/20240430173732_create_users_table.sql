CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR(255) NOT NULL,
  discord_id VARCHAR(255) UNIQUE,
  discord_name VARCHAR(255),
  twitch_id VARCHAR(255),
  twitch_name VARCHAR(255),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
