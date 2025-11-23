-- Cleanup script for Document tables
DROP TABLE IF EXISTS person_document_audit CASCADE;
DROP TABLE IF EXISTS person_document CASCADE;
DROP TYPE IF EXISTS document_type CASCADE;
DROP TYPE IF EXISTS document_status CASCADE;