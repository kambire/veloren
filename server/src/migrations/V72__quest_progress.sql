-- Misiones de World of Azeria: progreso de las misiones en curso y lista de
-- misiones completadas de cada personaje, guardado como JSON
-- (`common::quest::ActiveQuests`).
CREATE TABLE "quest_progress" (
    "character_id" INT NOT NULL,
    "data" TEXT NOT NULL,
    PRIMARY KEY("character_id"),
    FOREIGN KEY("character_id") REFERENCES "character"("character_id")
);
