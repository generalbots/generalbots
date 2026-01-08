-- Learn Module Database Migration
-- Learning Management System (LMS) for General Bots

-- ============================================================================
-- CATEGORIES TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    color TEXT,
    parent_id UUID REFERENCES learn_categories(id) ON DELETE SET NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_learn_categories_parent ON learn_categories(parent_id);
CREATE INDEX idx_learn_categories_sort ON learn_categories(sort_order);

-- Insert default categories
INSERT INTO learn_categories (name, description, icon, color, sort_order) VALUES
    ('compliance', 'Treinamentos de Compliance e Conformidade', '📋', '#3b82f6', 1),
    ('security', 'Segurança da Informação e LGPD', '🔒', '#ef4444', 2),
    ('onboarding', 'Integração de Novos Colaboradores', '🚀', '#10b981', 3),
    ('skills', 'Desenvolvimento de Habilidades', '💡', '#f59e0b', 4),
    ('leadership', 'Liderança e Gestão', '👔', '#8b5cf6', 5),
    ('technical', 'Treinamentos Técnicos', '⚙️', '#6366f1', 6),
    ('soft-skills', 'Soft Skills e Comunicação', '💬', '#ec4899', 7),
    ('wellness', 'Bem-estar e Qualidade de Vida', '🧘', '#14b8a6', 8);

-- ============================================================================
-- COURSES TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_courses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    title TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL DEFAULT 'skills',
    difficulty TEXT NOT NULL DEFAULT 'beginner',
    duration_minutes INTEGER NOT NULL DEFAULT 0,
    thumbnail_url TEXT,
    is_mandatory BOOLEAN NOT NULL DEFAULT FALSE,
    due_days INTEGER,
    is_published BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_difficulty CHECK (difficulty IN ('beginner', 'intermediate', 'advanced'))
);

CREATE INDEX idx_learn_courses_org ON learn_courses(organization_id);
CREATE INDEX idx_learn_courses_category ON learn_courses(category);
CREATE INDEX idx_learn_courses_mandatory ON learn_courses(is_mandatory);
CREATE INDEX idx_learn_courses_published ON learn_courses(is_published);
CREATE INDEX idx_learn_courses_created ON learn_courses(created_at DESC);

-- ============================================================================
-- LESSONS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_lessons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL REFERENCES learn_courses(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT,
    content_type TEXT NOT NULL DEFAULT 'text',
    lesson_order INTEGER NOT NULL DEFAULT 1,
    duration_minutes INTEGER NOT NULL DEFAULT 0,
    video_url TEXT,
    attachments JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_content_type CHECK (content_type IN ('text', 'video', 'pdf', 'slides', 'interactive'))
);

CREATE INDEX idx_learn_lessons_course ON learn_lessons(course_id);
CREATE INDEX idx_learn_lessons_order ON learn_lessons(course_id, lesson_order);

-- ============================================================================
-- QUIZZES TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_quizzes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lesson_id UUID REFERENCES learn_lessons(id) ON DELETE CASCADE,
    course_id UUID NOT NULL REFERENCES learn_courses(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    passing_score INTEGER NOT NULL DEFAULT 70,
    time_limit_minutes INTEGER,
    max_attempts INTEGER,
    questions JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_passing_score CHECK (passing_score >= 0 AND passing_score <= 100)
);

CREATE INDEX idx_learn_quizzes_course ON learn_quizzes(course_id);
CREATE INDEX idx_learn_quizzes_lesson ON learn_quizzes(lesson_id);

-- ============================================================================
-- USER PROGRESS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_user_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    course_id UUID NOT NULL REFERENCES learn_courses(id) ON DELETE CASCADE,
    lesson_id UUID REFERENCES learn_lessons(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'not_started',
    quiz_score INTEGER,
    quiz_attempts INTEGER NOT NULL DEFAULT 0,
    time_spent_minutes INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_status CHECK (status IN ('not_started', 'in_progress', 'completed', 'failed')),
    CONSTRAINT valid_quiz_score CHECK (quiz_score IS NULL OR (quiz_score >= 0 AND quiz_score <= 100))
);

CREATE INDEX idx_learn_progress_user ON learn_user_progress(user_id);
CREATE INDEX idx_learn_progress_course ON learn_user_progress(course_id);
CREATE INDEX idx_learn_progress_user_course ON learn_user_progress(user_id, course_id);
CREATE INDEX idx_learn_progress_status ON learn_user_progress(status);
CREATE INDEX idx_learn_progress_last_accessed ON learn_user_progress(last_accessed_at DESC);

-- Unique constraint for course-level progress (where lesson_id is null)
CREATE UNIQUE INDEX idx_learn_progress_user_course_unique
    ON learn_user_progress(user_id, course_id)
    WHERE lesson_id IS NULL;

-- Unique constraint for lesson-level progress
CREATE UNIQUE INDEX idx_learn_progress_user_lesson_unique
    ON learn_user_progress(user_id, lesson_id)
    WHERE lesson_id IS NOT NULL;

-- ============================================================================
-- COURSE ASSIGNMENTS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_course_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL REFERENCES learn_courses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    assigned_by UUID,
    due_date TIMESTAMPTZ,
    is_mandatory BOOLEAN NOT NULL DEFAULT TRUE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    reminder_sent BOOLEAN NOT NULL DEFAULT FALSE,
    reminder_sent_at TIMESTAMPTZ
);

CREATE INDEX idx_learn_assignments_user ON learn_course_assignments(user_id);
CREATE INDEX idx_learn_assignments_course ON learn_course_assignments(course_id);
CREATE INDEX idx_learn_assignments_due ON learn_course_assignments(due_date);
CREATE INDEX idx_learn_assignments_pending ON learn_course_assignments(user_id, completed_at)
    WHERE completed_at IS NULL;
CREATE INDEX idx_learn_assignments_overdue ON learn_course_assignments(due_date)
    WHERE completed_at IS NULL AND due_date IS NOT NULL;

-- Unique constraint to prevent duplicate assignments
CREATE UNIQUE INDEX idx_learn_assignments_unique ON learn_course_assignments(course_id, user_id);

-- ============================================================================
-- CERTIFICATES TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    course_id UUID NOT NULL REFERENCES learn_courses(id) ON DELETE CASCADE,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    score INTEGER NOT NULL,
    certificate_url TEXT,
    verification_code TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ,

    CONSTRAINT valid_cert_score CHECK (score >= 0 AND score <= 100)
);

CREATE INDEX idx_learn_certificates_user ON learn_certificates(user_id);
CREATE INDEX idx_learn_certificates_course ON learn_certificates(course_id);
CREATE INDEX idx_learn_certificates_verification ON learn_certificates(verification_code);
CREATE INDEX idx_learn_certificates_issued ON learn_certificates(issued_at DESC);

-- Unique constraint to prevent duplicate certificates
CREATE UNIQUE INDEX idx_learn_certificates_unique ON learn_certificates(user_id, course_id);

-- ============================================================================
-- QUIZ ATTEMPTS TABLE (for detailed tracking)
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_quiz_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quiz_id UUID NOT NULL REFERENCES learn_quizzes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    attempt_number INTEGER NOT NULL DEFAULT 1,
    score INTEGER NOT NULL,
    max_score INTEGER NOT NULL,
    passed BOOLEAN NOT NULL,
    time_taken_minutes INTEGER,
    answers JSONB NOT NULL DEFAULT '{}',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_learn_quiz_attempts_quiz ON learn_quiz_attempts(quiz_id);
CREATE INDEX idx_learn_quiz_attempts_user ON learn_quiz_attempts(user_id);
CREATE INDEX idx_learn_quiz_attempts_user_quiz ON learn_quiz_attempts(user_id, quiz_id);

-- ============================================================================
-- LEARNING PATHS TABLE (for structured learning journeys)
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_paths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    name TEXT NOT NULL,
    description TEXT,
    thumbnail_url TEXT,
    is_published BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_learn_paths_org ON learn_paths(organization_id);
CREATE INDEX idx_learn_paths_published ON learn_paths(is_published);

-- ============================================================================
-- LEARNING PATH COURSES (junction table)
-- ============================================================================
CREATE TABLE IF NOT EXISTS learn_path_courses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path_id UUID NOT NULL REFERENCES learn_paths(id) ON DELETE CASCADE,
    course_id UUID NOT NULL REFERENCES learn_courses(id) ON DELETE CASCADE,
    course_order INTEGER NOT NULL DEFAULT 1,
    is_required BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_learn_path_courses_path ON learn_path_courses(path_id);
CREATE INDEX idx_learn_path_courses_course ON learn_path_courses(course_id);
CREATE UNIQUE INDEX idx_learn_path_courses_unique ON learn_path_courses(path_id, course_id);

-- ============================================================================
-- SAMPLE DATA FOR DEMONSTRATION
-- ============================================================================

-- Sample mandatory compliance course
INSERT INTO learn_courses (id, title, description, category, difficulty, duration_minutes, is_mandatory, due_days, is_published) VALUES
    ('11111111-1111-1111-1111-111111111111',
     'LGPD - Lei Geral de Proteção de Dados',
     'Treinamento obrigatório sobre a Lei Geral de Proteção de Dados Pessoais. Aprenda os conceitos fundamentais, direitos dos titulares e obrigações da empresa.',
     'compliance', 'beginner', 45, TRUE, 30, TRUE),
    ('22222222-2222-2222-2222-222222222222',
     'Código de Conduta e Ética',
     'Conheça nosso código de conduta e ética empresarial. Este treinamento é obrigatório para todos os colaboradores.',
     'compliance', 'beginner', 30, TRUE, 14, TRUE),
    ('33333333-3333-3333-3333-333333333333',
     'Segurança da Informação',
     'Aprenda as melhores práticas de segurança da informação para proteger dados da empresa e de clientes.',
     'security', 'intermediate', 60, TRUE, 30, TRUE),
    ('44444444-4444-4444-4444-444444444444',
     'Integração de Novos Colaboradores',
     'Bem-vindo à empresa! Este curso apresenta nossa cultura, valores e processos fundamentais.',
     'onboarding', 'beginner', 90, FALSE, NULL, TRUE),
    ('55555555-5555-5555-5555-555555555555',
     'Comunicação Efetiva',
     'Desenvolva habilidades de comunicação profissional para trabalhar melhor em equipe.',
     'soft-skills', 'intermediate', 40, FALSE, NULL, TRUE);

-- Sample lessons for LGPD course
INSERT INTO learn_lessons (course_id, title, content, content_type, lesson_order, duration_minutes) VALUES
    ('11111111-1111-1111-1111-111111111111', 'Introdução à LGPD',
     '<h2>O que é a LGPD?</h2><p>A Lei Geral de Proteção de Dados (Lei nº 13.709/2018) é a legislação brasileira que regula as atividades de tratamento de dados pessoais.</p><h3>Objetivos da LGPD</h3><ul><li>Proteger os direitos fundamentais de liberdade e privacidade</li><li>Estabelecer regras claras sobre tratamento de dados pessoais</li><li>Fomentar o desenvolvimento econômico e tecnológico</li></ul>',
     'text', 1, 10),
    ('11111111-1111-1111-1111-111111111111', 'Conceitos Fundamentais',
     '<h2>Principais Conceitos</h2><h3>Dados Pessoais</h3><p>Informação relacionada a pessoa natural identificada ou identificável.</p><h3>Dados Sensíveis</h3><p>Dados sobre origem racial, convicção religiosa, opinião política, filiação sindical, dados de saúde, vida sexual, dados genéticos ou biométricos.</p><h3>Tratamento de Dados</h3><p>Toda operação realizada com dados pessoais: coleta, armazenamento, uso, compartilhamento, exclusão, etc.</p>',
     'text', 2, 15),
    ('11111111-1111-1111-1111-111111111111', 'Direitos dos Titulares',
     '<h2>Direitos Garantidos pela LGPD</h2><ul><li><strong>Confirmação e acesso:</strong> saber se seus dados são tratados e ter acesso a eles</li><li><strong>Correção:</strong> corrigir dados incompletos, inexatos ou desatualizados</li><li><strong>Anonimização e exclusão:</strong> solicitar a anonimização ou eliminação de dados desnecessários</li><li><strong>Portabilidade:</strong> transferir seus dados para outro fornecedor</li><li><strong>Revogação do consentimento:</strong> revogar o consentimento a qualquer momento</li></ul>',
     'text', 3, 10),
    ('11111111-1111-1111-1111-111111111111', 'Responsabilidades da Empresa',
     '<h2>Nossas Responsabilidades</h2><p>Como colaboradores, temos o dever de:</p><ul><li>Tratar dados pessoais apenas para finalidades legítimas</li><li>Manter a confidencialidade das informações</li><li>Reportar incidentes de segurança</li><li>Seguir os procedimentos internos de proteção de dados</li></ul>',
     'text', 4, 10);

-- Sample quiz for LGPD course
INSERT INTO learn_quizzes (course_id, title, passing_score, time_limit_minutes, questions) VALUES
    ('11111111-1111-1111-1111-111111111111', 'Avaliação - LGPD', 70, 15, '[
        {
            "id": "q1",
            "text": "O que significa LGPD?",
            "question_type": "single_choice",
            "options": [
                {"text": "Lei Geral de Proteção de Dados", "is_correct": true},
                {"text": "Lei Governamental de Privacidade Digital", "is_correct": false},
                {"text": "Legislação Geral de Proteção Digital", "is_correct": false},
                {"text": "Lei de Garantia e Proteção de Dados", "is_correct": false}
            ],
            "correct_answers": [0],
            "explanation": "LGPD significa Lei Geral de Proteção de Dados (Lei nº 13.709/2018).",
            "points": 10
        },
        {
            "id": "q2",
            "text": "Quais são considerados dados sensíveis pela LGPD?",
            "question_type": "multiple_choice",
            "options": [
                {"text": "Dados de saúde", "is_correct": true},
                {"text": "Nome e CPF", "is_correct": false},
                {"text": "Origem racial ou étnica", "is_correct": true},
                {"text": "Endereço de e-mail", "is_correct": false},
                {"text": "Convicção religiosa", "is_correct": true}
            ],
            "correct_answers": [0, 2, 4],
            "explanation": "Dados sensíveis incluem: origem racial/étnica, convicção religiosa, opinião política, dados de saúde, vida sexual, dados genéticos ou biométricos.",
            "points": 20
        },
        {
            "id": "q3",
            "text": "O titular dos dados tem direito de solicitar a exclusão de seus dados pessoais.",
            "question_type": "true_false",
            "options": [
                {"text": "Verdadeiro", "is_correct": true},
                {"text": "Falso", "is_correct": false}
            ],
            "correct_answers": [0],
            "explanation": "Sim, o direito à eliminação de dados é um dos direitos garantidos pela LGPD.",
            "points": 10
        },
        {
            "id": "q4",
            "text": "O que é tratamento de dados segundo a LGPD?",
            "question_type": "single_choice",
            "options": [
                {"text": "Apenas o armazenamento de dados", "is_correct": false},
                {"text": "Toda operação realizada com dados pessoais", "is_correct": true},
                {"text": "Somente a coleta de dados", "is_correct": false},
                {"text": "A venda de dados pessoais", "is_correct": false}
            ],
            "correct_answers": [1],
            "explanation": "Tratamento é toda operação: coleta, produção, recepção, classificação, utilização, acesso, reprodução, transmissão, distribuição, processamento, arquivamento, armazenamento, eliminação, etc.",
            "points": 10
        },
        {
            "id": "q5",
            "text": "Qual é o prazo para a empresa responder a uma solicitação do titular dos dados?",
            "question_type": "single_choice",
            "options": [
                {"text": "Imediatamente", "is_correct": false},
                {"text": "Em até 15 dias", "is_correct": true},
                {"text": "Em até 30 dias", "is_correct": false},
                {"text": "Em até 90 dias", "is_correct": false}
            ],
            "correct_answers": [1],
            "explanation": "A LGPD estabelece que a empresa deve responder às solicitações dos titulares em até 15 dias.",
            "points": 10
        }
    ]');

-- Add lessons for other courses
INSERT INTO learn_lessons (course_id, title, content_type, lesson_order, duration_minutes) VALUES
    ('22222222-2222-2222-2222-222222222222', 'Nossa Missão e Valores', 'text', 1, 10),
    ('22222222-2222-2222-2222-222222222222', 'Conduta Profissional', 'text', 2, 10),
    ('22222222-2222-2222-2222-222222222222', 'Canais de Denúncia', 'text', 3, 10),
    ('33333333-3333-3333-3333-333333333333', 'Ameaças Digitais', 'text', 1, 15),
    ('33333333-3333-3333-3333-333333333333', 'Senhas Seguras', 'text', 2, 15),
    ('33333333-3333-3333-3333-333333333333', 'Phishing e Engenharia Social', 'text', 3, 15),
    ('33333333-3333-3333-3333-333333333333', 'Políticas de Segurança', 'text', 4, 15);

-- ============================================================================
-- TRIGGERS FOR UPDATED_AT
-- ============================================================================

CREATE OR REPLACE FUNCTION update_learn_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_learn_courses_updated_at
    BEFORE UPDATE ON learn_courses
    FOR EACH ROW
    EXECUTE FUNCTION update_learn_updated_at();

CREATE TRIGGER trigger_learn_lessons_updated_at
    BEFORE UPDATE ON learn_lessons
    FOR EACH ROW
    EXECUTE FUNCTION update_learn_updated_at();

CREATE TRIGGER trigger_learn_quizzes_updated_at
    BEFORE UPDATE ON learn_quizzes
    FOR EACH ROW
    EXECUTE FUNCTION update_learn_updated_at();

CREATE TRIGGER trigger_learn_paths_updated_at
    BEFORE UPDATE ON learn_paths
    FOR EACH ROW
    EXECUTE FUNCTION update_learn_updated_at();
