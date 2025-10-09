-- Create users table.
CREATE TABLE users (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    -- Stores hashed passwords for email/password users
    password_hash VARCHAR(255),
    -- e.g., 'google', 'github', 'facebook'
    oauth_provider VARCHAR(50),
    -- Unique ID provided by the OAuth provider
    oauth_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
