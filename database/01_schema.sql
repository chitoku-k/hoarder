CREATE TABLE "external_services" (
    "id" uuid DEFAULT uuid_generate_v4(),
    "slug" text NOT NULL,
    "name" text NOT NULL,
    PRIMARY KEY ("id"),
    UNIQUE ("slug")
);
CREATE TABLE "sources" (
    "id" uuid DEFAULT uuid_generate_v4(),
    "external_service_id" uuid NOT NULL REFERENCES "external_services" ("id") ON DELETE CASCADE,
    "external_metadata" jsonb NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id"),
    UNIQUE ("external_service_id", "external_metadata")
);
CREATE TABLE "media" (
    "id" uuid DEFAULT uuid_generate_v4(),
    "created_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
CREATE TABLE "media_sources" (
    "medium_id" uuid NOT NULL REFERENCES "media" ("id") ON DELETE CASCADE,
    "source_id" uuid NOT NULL REFERENCES "sources" ("id") ON DELETE CASCADE,
    UNIQUE ("medium_id", "source_id")
);
CREATE TABLE "replicas" (
    "id" uuid DEFAULT uuid_generate_v4(),
    "medium_id" uuid NOT NULL REFERENCES "media" ("id") ON DELETE CASCADE,
    "display_order" integer CHECK ("display_order" IS NULL OR "display_order" > 0),
    "original_url" text NOT NULL,
    "mime_type" text NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id"),
    UNIQUE ("medium_id", "display_order"),
    UNIQUE ("original_url")
);
CREATE TABLE "thumbnails" (
    "id" uuid DEFAULT uuid_generate_v4(),
    "replica_id" uuid NOT NULL REFERENCES "replicas" ("id") ON DELETE CASCADE,
    "data" bytea NOT NULL,
    "width" integer NOT NULL,
    "height" integer NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id"),
    UNIQUE ("replica_id")
);
CREATE TABLE "tags" (
    "id" uuid DEFAULT uuid_generate_v4(),
    "name" text NOT NULL,
    "kana" text NOT NULL,
    "aliases" text[] NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
CREATE TABLE "tag_paths" (
    "ancestor_id" uuid NOT NULL REFERENCES "tags" ("id") ON DELETE CASCADE,
    "descendant_id" uuid NOT NULL REFERENCES "tags" ("id") ON DELETE CASCADE,
    "distance" integer NOT NULL,
    UNIQUE ("descendant_id", "distance"),
    CHECK (
        (
            ("ancestor_id" = '00000000-0000-0000-0000-000000000000' AND "descendant_id" = '00000000-0000-0000-0000-000000000000') OR
            ("descendant_id" <> '00000000-0000-0000-0000-000000000000')
        ) AND (
            ("distance" > 0 AND "ancestor_id" <> "descendant_id") OR
            ("distance" = 0 AND "ancestor_id" = "descendant_id")
        )
    )
);
CREATE TABLE "tag_types" (
    "id" uuid DEFAULT uuid_generate_v4(),
    "slug" text NOT NULL,
    "name" text NOT NULL,
    PRIMARY KEY ("id"),
    UNIQUE ("slug")
);
CREATE TABLE "media_tags" (
    "medium_id" uuid NOT NULL REFERENCES "media" ("id") ON DELETE CASCADE,
    "tag_id" uuid NOT NULL REFERENCES "tags" ("id") ON DELETE CASCADE,
    "tag_type_id" uuid NOT NULL REFERENCES "tag_types" ("id") ON DELETE CASCADE,
    UNIQUE ("medium_id", "tag_id", "tag_type_id")
);
CREATE TABLE "jobs" (
    "id" uuid DEFAULT uuid_generate_v4(),
    "content" jsonb NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
CREATE TABLE "job_runs" (
    "job_id" uuid NOT NULL REFERENCES "jobs" ("id") ON DELETE CASCADE,
    "phase" text NOT NULL,
    "message" text NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE ("job_id", "phase")
);
