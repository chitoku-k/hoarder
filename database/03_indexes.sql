CREATE INDEX ON "media" ("created_at", "id");
CREATE INDEX ON "replicas" ("medium_id");
CREATE INDEX ON "tags" ("aliases");
CREATE INDEX ON "tags" ("kana", "id");
CREATE INDEX ON "tag_paths" ("distance", "ancestor_id", "descendant_id");
