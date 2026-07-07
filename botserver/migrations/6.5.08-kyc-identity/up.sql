DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS identity_profiles (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        person_id UUID NOT NULL,
        legal_name TEXT NOT NULL,
        tax_id VARCHAR(50) NOT NULL,
        date_of_birth DATE,
        nationality VARCHAR(50),
        email VARCHAR(255),
        phone VARCHAR(50),
        address JSONB,
        risk_score INTEGER,
        kyc_status VARCHAR(30) NOT NULL DEFAULT 'pending',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_id_profile_bot ON identity_profiles(bot_id);
    CREATE INDEX IF NOT EXISTS idx_id_profile_tax_id ON identity_profiles(tax_id);
    CREATE INDEX IF NOT EXISTS idx_id_profile_status ON identity_profiles(kyc_status);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating identity_profiles table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS identity_faces (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        profile_id UUID NOT NULL REFERENCES identity_profiles(id),
        photo_url TEXT NOT NULL,
        embedding JSONB,
        quality_score NUMERIC(5,4),
        is_primary BOOLEAN NOT NULL DEFAULT false,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_id_face_profile ON identity_faces(profile_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating identity_faces table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS identity_documents (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        profile_id UUID NOT NULL REFERENCES identity_profiles(id),
        document_type VARCHAR(30) NOT NULL,
        document_number VARCHAR(100) NOT NULL,
        issuing_country VARCHAR(10),
        issue_date DATE,
        expiry_date DATE,
        front_image_url TEXT,
        back_image_url TEXT,
        selfie_image_url TEXT,
        ocr_data JSONB,
        verification_status VARCHAR(30) NOT NULL DEFAULT 'pending',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_id_doc_profile ON identity_documents(profile_id);
    CREATE INDEX IF NOT EXISTS idx_id_doc_type ON identity_documents(document_type);
    CREATE INDEX IF NOT EXISTS idx_id_doc_status ON identity_documents(verification_status);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating identity_documents table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS identity_signatures (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        profile_id UUID NOT NULL REFERENCES identity_profiles(id),
        document_id UUID NOT NULL REFERENCES identity_documents(id),
        signature_data TEXT NOT NULL,
        signature_image_url TEXT,
        ip_address VARCHAR(64),
        user_agent TEXT,
        signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_id_sig_profile ON identity_signatures(profile_id);
    CREATE INDEX IF NOT EXISTS idx_id_sig_doc ON identity_signatures(document_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating identity_signatures table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS identity_signed_documents (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        signature_id UUID NOT NULL REFERENCES identity_signatures(id),
        document_hash VARCHAR(128) NOT NULL,
        document_name TEXT NOT NULL,
        signature_algorithm VARCHAR(30) NOT NULL,
        signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_id_signed_doc_sig ON identity_signed_documents(signature_id);
    CREATE INDEX IF NOT EXISTS idx_id_signed_doc_hash ON identity_signed_documents(document_hash);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating identity_signed_documents table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS identity_kyc_workflows (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        profile_id UUID NOT NULL REFERENCES identity_profiles(id),
        workflow_name VARCHAR(100) NOT NULL,
        current_step VARCHAR(100) NOT NULL,
        steps_completed JSONB NOT NULL DEFAULT '[]',
        total_steps INTEGER NOT NULL,
        status VARCHAR(30) NOT NULL DEFAULT 'in_progress',
        started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        completed_at TIMESTAMPTZ
    );

    CREATE INDEX IF NOT EXISTS idx_id_kyc_profile ON identity_kyc_workflows(profile_id);
    CREATE INDEX IF NOT EXISTS idx_id_kyc_status ON identity_kyc_workflows(status);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating identity_kyc_workflows table: %', SQLERRM;
END $$;
