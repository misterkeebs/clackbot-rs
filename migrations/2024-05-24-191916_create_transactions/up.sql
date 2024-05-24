CREATE TABLE transactions (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  description TEXT NOT NULL,
  clacks INTEGER NOT NULL DEFAULT 0,
  modified_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TRIGGER update_transaction_mod_time
BEFORE UPDATE ON transactions
FOR EACH ROW
EXECUTE PROCEDURE update_modified_column();
