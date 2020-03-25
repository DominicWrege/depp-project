create extension "pgcrypto";

CREATE TYPE script_type AS ENUM(
    'PowerShell',
	'Batch',
	'Python3',
	'Bash',
	'Shell',
	'Awk',
	'Sed'
);

CREATE TABLE exercise(
    id SERIAL PRIMARY KEY,
    description text NOT NULL
);


CREATE TABLE assignment (
    id SERIAL PRIMARY KEY,
    uuid uuid not null default gen_random_uuid(),
    assignment_name text not null default '',
    script_type script_type not null,
    active boolean not null default true,
    include_files bytea, -- zip of files
    solution text not null,
    args text[] not null default '{}',
    exercise_id INTEGER REFERENCES exercise(id) ON DELETE CASCADE NOT NULL,
    description text not null default ''
);

INSERT INTO exercise(description) VALUES ('Praktikum 01');
