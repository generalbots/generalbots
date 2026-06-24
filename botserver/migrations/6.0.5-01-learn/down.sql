-- Learn Module Database Migration - Rollback
-- Removes all Learn module tables and data

-- Drop triggers first
DROP TRIGGER IF EXISTS trigger_learn_paths_updated_at ON learn_paths;
DROP TRIGGER IF EXISTS trigger_learn_quizzes_updated_at ON learn_quizzes;
DROP TRIGGER IF EXISTS trigger_learn_lessons_updated_at ON learn_lessons;
DROP TRIGGER IF EXISTS trigger_learn_courses_updated_at ON learn_courses;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_learn_updated_at();

-- Drop tables in reverse order of creation (respecting foreign key constraints)
DROP TABLE IF EXISTS learn_path_courses CASCADE;
DROP TABLE IF EXISTS learn_paths CASCADE;
DROP TABLE IF EXISTS learn_quiz_attempts CASCADE;
DROP TABLE IF EXISTS learn_certificates CASCADE;
DROP TABLE IF EXISTS learn_course_assignments CASCADE;
DROP TABLE IF EXISTS learn_user_progress CASCADE;
DROP TABLE IF EXISTS learn_quizzes CASCADE;
DROP TABLE IF EXISTS learn_lessons CASCADE;
DROP TABLE IF EXISTS learn_courses CASCADE;
DROP TABLE IF EXISTS learn_categories CASCADE;
