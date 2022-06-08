INSERT INTO "tags"
    ("id", "name", "kana", "aliases", "created_at", "updated_at")
VALUES
    ('00000000-0000-0000-0000-000000000000', 'root', 'root', '{}', '1970-01-01 00:00:00', '1970-01-01 00:00:00');

INSERT INTO "tag_paths"
    ("ancestor_id", "descendant_id", "distance")
VALUES
    ('00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000', 0);
