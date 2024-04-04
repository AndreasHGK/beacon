create table files (
    file_id bigint not null primary key,
    file_name text not null,    

    -- The size of the file in bytes.
    file_size bigint not null constraint positive_size check (file_size >= 0),
    upload_date timestamptz not null
);
