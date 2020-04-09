
CREATE TYPE regex_mode AS ENUM(
    'UnknownRegex',
    'Stdout',
    'ScriptContent'
);

CREATE TYPE sort_stdout_by AS ENUM(
    'UnknownSort',
    'Asc',
    'Desc'
);


ALTER TABLE assignment
    ADD COLUMN compare_fs_solution boolean not null default true,
    ADD COLUMN compare_stdout_solution boolean not null default true,
    ADD COLUMN custom_script text,
    ADD COLUMN regex text,
    ADD COLUMN regex_check_mode regex_mode not null default 'UnknownRegex',
    ADD COLUMN sort_stdout sort_stdout_by not null default 'UnknownSort';



