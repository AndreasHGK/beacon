create table sessions (
    -- The token is a random sequence of bytes. 128 bytes is the minimum recommended by OWASP:
    -- https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#session-id-length
    token bytea primary key constraint min_token_len check (length(token) >= 16),
    user_id uuid not null references users(user_id) on delete cascade,
    -- The token should only be considered valid if the current time is between the following two
    -- timestamps.
    issued_at timestamptz not null,
    expires_on timestamptz not null
);
