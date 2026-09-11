CREATE TABLE records (
    id INTEGER PRIMARY KEY,
    value TEXT NOT NULL UNIQUE
);
CREATE TABLE children (
    id INTEGER PRIMARY KEY,
    record_id INTEGER NOT NULL REFERENCES records(id)
);
