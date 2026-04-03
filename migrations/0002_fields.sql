-- @up

CREATE TYPE field_value_type AS ENUM ('text', 'enum', 'bool', 'int', 'decimal');

CREATE TABLE field_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key VARCHAR(100) NOT NULL,
    value_type field_value_type NOT NULL DEFAULT 'text',
    color VARCHAR(7),
    options TEXT[],
    position INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, key)
);

SELECT forge_enable_reactivity('field_definitions');

CREATE TABLE task_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    field_id UUID NOT NULL REFERENCES field_definitions(id) ON DELETE CASCADE,
    value TEXT NOT NULL DEFAULT '',
    UNIQUE(task_id, field_id)
);

SELECT forge_enable_reactivity('task_fields');

-- @down

SELECT forge_disable_reactivity('task_fields');
DROP TABLE IF EXISTS task_fields;
SELECT forge_disable_reactivity('field_definitions');
DROP TABLE IF EXISTS field_definitions;
DROP TYPE IF EXISTS field_value_type;
