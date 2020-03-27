CREATE OR REPLACE FUNCTION cleanup_init_exercise() RETURNS VOID AS $$
BEGIN
    FOR i IN 3..10 LOOP
            DELETE FROM exercise WHERE description = concat('Praktikum', to_char(i, '09'));
        END LOOP;
END;
$$ LANGUAGE plpgsql;

select "cleanup_init_exercise"();

DROP FUNCTION IF EXISTS init_exercise;
DROP FUNCTION IF EXISTS cleanup_init_exercise;

