CREATE OR REPLACE VIEW assignment_exercise
AS Select assignment_name as name, script_type, active, solution, args, e.description, notes as exercise_name, include_files
from assignment a, exercise e WHERE a.exercise_id = e.id;
