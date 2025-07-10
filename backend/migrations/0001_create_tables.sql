-- Add messages table
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
); 