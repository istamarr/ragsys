-- ============================================
-- SurrealDB Queries for User Credentials
-- ============================================
-- Run these directly in SurrealDB console or via HTTP API

-- ============================================
-- 1. Connect to Namespace and Database
-- ============================================
USE NS pgd_ml_nmspace DB pgd_db;

-- ============================================
-- 2. Create user_credentials Table
-- ============================================
-- SurrealDB creates tables automatically when you insert data
-- But you can define schema if needed:

DEFINE TABLE user_credentials SCHEMAFULL;

DEFINE FIELD key ON TABLE user_credentials TYPE string;
DEFINE FIELD email ON TABLE user_credentials TYPE string;
DEFINE FIELD password ON TABLE user_credentials TYPE string;
DEFINE FIELD role ON TABLE user_credentials TYPE string;
DEFINE FIELD create_date ON TABLE user_credentials TYPE datetime;
DEFINE FIELD update_date ON TABLE user_credentials TYPE datetime;
DEFINE FIELD create_by ON TABLE user_credentials TYPE string;
DEFINE FIELD update_by ON TABLE user_credentials TYPE string;

-- ============================================
-- 3. Insert User Records (Password must be AES-256-GCM encrypted)
-- ============================================

-- Insert user 1
CREATE user_credentials CONTENT {
    key: 'user1',
    email: 'user1@example.com',
    password: 'ENCRYPTED_PASSWORD_BASE64_HERE',
    role: 'DEV',
    create_date: time::now(),
    update_date: time::now(),
    create_by: 'system',
    update_by: 'system'
};

-- Insert user 2
CREATE user_credentials CONTENT {
    key: 'user2',
    email: 'user2@example.com',
    password: 'ENCRYPTED_PASSWORD_BASE64_HERE',
    role: 'ADMIN',
    create_date: time::now(),
    update_date: time::now(),
    create_by: 'system',
    update_by: 'system'
};

-- ============================================
-- 4. Select/Query Users
-- ============================================

-- Select all users
SELECT * FROM user_credentials;

-- Select specific user by email
SELECT * FROM user_credentials WHERE email = 'user1@example.com';

-- Select users by role
SELECT * FROM user_credentials WHERE role = 'DEV';

-- ============================================
-- 5. Update User
-- ============================================

-- Update user password
UPDATE user_credentials SET 
    password = 'NEW_ENCRYPTED_PASSWORD_BASE64_HERE',
    update_date = time::now(),
    update_by = 'admin'
WHERE email = 'user1@example.com';

-- Update user role
UPDATE user_credentials SET 
    role = 'ADMIN',
    update_date = time::now(),
    update_by: 'admin'
WHERE email = 'user1@example.com';

-- ============================================
-- 6. Delete User
-- ============================================

-- Delete specific user
DELETE user_credentials WHERE email = 'user1@example.com';

-- ============================================
-- 7. Create todo Table (for todo_repository)
-- ============================================

DEFINE TABLE todo SCHEMAFULL;

DEFINE FIELD title ON TABLE todo TYPE string;
DEFINE FIELD content ON TABLE todo TYPE string;
DEFINE FIELD completed ON TABLE todo TYPE bool;
DEFINE FIELD createdAt ON TABLE todo TYPE datetime;
DEFINE FIELD updatedAt ON TABLE todo TYPE datetime;

-- Insert todo
CREATE todo CONTENT {
    title: 'Sample Todo',
    content: 'This is a sample todo item',
    completed: false,
    createdAt: time::now(),
    updatedAt: time::now()
};

-- ============================================
-- 8. Create log_data Table (for jobcron)
-- ============================================

DEFINE TABLE log_data SCHEMAFULL;

DEFINE FIELD message ON TABLE log_data TYPE string;
DEFINE FIELD level ON TABLE log_data TYPE string;
DEFINE FIELD timestamp ON TABLE log_data TYPE datetime;

-- Insert log
CREATE log_data CONTENT {
    message: 'Sample log message',
    level: 'INFO',
    timestamp: time::now()
};

-- ============================================
-- 9. Common Utility Queries
-- ============================================

-- Count users
SELECT count() FROM user_credentials;

-- List all tables
SHOW TABLES;

-- Describe table structure
INFO FOR TABLE user_credentials;

-- ============================================
-- 10. Authentication Queries
-- ============================================

-- Check if user exists (for login validation)
SELECT * FROM user_credentials WHERE email = 'user@example.com';

-- ============================================
-- Note: Password Encryption
-- ============================================
-- Passwords must be encrypted using AES-256-GCM before insertion
-- Use the encrypt_data() function from secureUtils.rs
-- Key: 32 bytes (currently [0u8; 32])
-- Nonce: 12 bytes (randomly generated)
-- Encoding: Base64
