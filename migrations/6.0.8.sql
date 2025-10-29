ALTER TABLE bot_configuration
ADD CONSTRAINT bot_configuration_config_key_unique UNIQUE (config_key);
