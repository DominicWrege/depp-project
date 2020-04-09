

CREATE TYPE regex_mode AS ENUM(
    'Stdout',
    'Script'
);


ALTER TABLE assignment
    ADD COLUMN compare_fs_solution boolean not null default true,
    ADD COLUMN compare_stdout_solution boolean not null default true,
    ADD COLUMN custom_script text,
    ADD COLUMN regex text,
    ADD COLUMN regex_check_mode regex_mode;
