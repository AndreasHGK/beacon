create table invites (
    invite text primary key,
    times_used int not null default 0
        constraint enforce_max_uses check (times_used <= max_uses),
    max_uses int not null,
    valid_until timestamptz not null,
    created_by uuid not null references users(user_id)
);

-- Each user now optionally stores the invite they used to create an account.
alter table users
    add column invite_used text
        references invites(invite)
        on delete set null;
