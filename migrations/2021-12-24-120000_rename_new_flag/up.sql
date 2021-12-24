ALTER TABLE metadata ADD edited BOOLEAN;
UPDATE metadata SET edited = false;
