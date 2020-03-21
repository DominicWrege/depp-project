CREATE OR REPLACE VIEW assignment_exercise
AS Select assignment_name as name, script_type, e.description as exercise_name, a.description
from assignment a, exercise e
WHERE a.exercise_id = e.id;
