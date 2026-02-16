ALTER TABLE subsystems
ADD COLUMN class_number INTEGER CHECK(class_number IN (1, 2, 3) OR class_number IS NULL);
