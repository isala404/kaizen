-- @up

CREATE TYPE attachment_display AS ENUM ('inline', 'attached');

CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    size_bytes BIGINT NOT NULL,
    storage_key TEXT NOT NULL,
    display attachment_display NOT NULL DEFAULT 'attached',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attachments_task ON attachments(task_id);

SELECT forge_enable_reactivity('attachments');

-- @down

SELECT forge_disable_reactivity('attachments');
DROP TABLE IF EXISTS attachments;
DROP TYPE IF EXISTS attachment_display;
