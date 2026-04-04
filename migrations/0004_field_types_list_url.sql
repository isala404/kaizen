-- @up

ALTER TYPE field_value_type ADD VALUE 'list';
ALTER TYPE field_value_type ADD VALUE 'url';

-- @down
-- PostgreSQL doesn't support removing enum values
