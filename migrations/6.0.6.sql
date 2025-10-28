-- Migration 6.0.6: Add LLM configuration defaults
-- Description: Configure local LLM server settings with model paths

-- Insert LLM configuration with defaults
INSERT INTO server_configuration (id, config_key, config_value, config_type, description) VALUES
    (gen_random_uuid()::text, 'LLM_LOCAL', 'true', 'boolean', 'Enable local LLM server'),
    (gen_random_uuid()::text, 'LLM_MODEL_PATH', 'botserver-stack/data/llm/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf', 'string', 'Path to LLM model file'),
    (gen_random_uuid()::text, 'LLM_URL', 'http://localhost:8081', 'string', 'Local LLM server URL'),
    (gen_random_uuid()::text, 'EMBEDDING_MODEL_PATH', 'botserver-stack/data/llm/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf', 'string', 'Path to embedding model file'),
    (gen_random_uuid()::text, 'EMBEDDING_URL', 'http://localhost:8082', 'string', 'Embedding server URL')
ON CONFLICT (config_key) DO NOTHING;
