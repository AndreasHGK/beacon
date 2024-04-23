create table users (
    user_id uuid primary key,
    username text not null unique,
    -- Argon2id password hash.
    password_hash text not null,
    created_at timestamptz not null default now(),
    -- If true, allow the user to manage the instance.
    is_admin bool not null default false
);

-- Create the default admin user.
insert into users (user_id, username, password_hash, is_admin) values (
    gen_random_uuid(),
    'admin',
    -- Default admin password: `please change me!`. This should, as the name suggests, be changed
    -- when deploying the service.
    '$argon2id$v=19$m=19456,t=2,p=1$amNZJvxxwV2qDbT2/48bqg$IVuv8DqwtCDCK3CPplhGoj3Tff7ocZBsc/fyPH0h4Q8',
    -- The admind user should have admin privileges.
    true
);

-- Files should not reference their uploader. If there are any existing files, the admin user is
-- used as a default.
alter table files add column uploader_id uuid references users(user_id);
update files set uploader_id = (select user_id from users limit 1);
alter table files alter column uploader_id set not null;
