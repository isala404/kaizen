-- @up

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

SELECT forge_enable_reactivity('users');

CREATE TYPE task_status AS ENUM (
    'inbox', 'up_next', 'in_progress', 'focused', 'paused', 'done', 'archived'
);

CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status task_status NOT NULL DEFAULT 'inbox',
    time_spent_secs BIGINT NOT NULL DEFAULT 0,
    position INT NOT NULL DEFAULT 0,
    due_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_user_status ON tasks(user_id, status);

SELECT forge_enable_reactivity('tasks');

-- @down

SELECT forge_disable_reactivity('tasks');
DROP TABLE IF EXISTS tasks;
SELECT forge_disable_reactivity('users');
DROP TABLE IF EXISTS users;
DROP TYPE IF EXISTS task_status;
