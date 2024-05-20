create table ssh_keys (
    user_id uuid not null references users on delete cascade,
    -- The fingerprint corresponding to the public key.
    public_key_fingerprint text not null,
    -- The OpenSSH encoded public key.
    public_key text not null,
    -- A user-defined display name for the key.
    name text not null,
    -- When was the key added?
    add_date timestamptz not null default now(),
    -- When was the last time the key was used?
    last_use timestamptz,

    primary key (user_id, public_key_fingerprint)
);
