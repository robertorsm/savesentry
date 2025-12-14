-- Migration V3: Add process_name column to game_profiles
-- Permite rastrear qual processo deve estar rodando antes de monitorar o arquivo

ALTER TABLE game_profiles ADD COLUMN process_name TEXT;

