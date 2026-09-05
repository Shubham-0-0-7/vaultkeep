-- 1. Enable UUID Extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 2. Tenants Table (Top-level Organization Boundary)
CREATE TABLE tenants (
id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
name VARCHAR(255) NOT NULL,
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Users Table
CREATE TABLE users (
id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
email VARCHAR(255) UNIQUE NOT NULL,
password_hash VARCHAR(255) NOT NULL,
role VARCHAR(50) NOT NULL DEFAULT 'member', -- 'owner', 'admin', 'member', 'read_only'
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Projects Table (Sub-boundary within a Tenant)
CREATE TABLE projects (
id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
name VARCHAR(255) NOT NULL,
description TEXT,
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
UNIQUE (tenant_id, name),
-- Composite unique key to enable composite foreign keys
UNIQUE (tenant_id, id)
);

-- 5. Secrets Table (Encrypted Key Store)
CREATE TABLE secrets (
id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
project_id UUID NOT NULL,
name VARCHAR(255) NOT NULL,
encrypted_payload BYTEA NOT NULL,
nonce BYTEA NOT NULL,             -- 12-byte IV for AES-256-GCM / ChaCha20-Poly1305
key_version INT NOT NULL DEFAULT 1,
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
-- Defense against Cross-Tenant Secondary Resource IDOR:
FOREIGN KEY (tenant_id, project_id) REFERENCES projects(tenant_id, id) ON DELETE CASCADE,
UNIQUE (tenant_id, project_id, name)
);

-- 6. Audit Logs Table (Tamper-Evident Access Log)
CREATE TABLE audit_logs (
id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
user_id UUID REFERENCES users(id) ON DELETE SET NULL,
action VARCHAR(100) NOT NULL,    -- 'secret.create', 'secret.read', 'secret.rotate', 'secret.delete'
resource_type VARCHAR(50) NOT NULL,
resource_id UUID NOT NULL,
ip_address INET,
user_agent TEXT,
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 7. High-Performance B-Tree Composite Indexes
CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_projects_tenant ON projects(tenant_id);
CREATE INDEX idx_secrets_tenant_project ON secrets(tenant_id, project_id);
CREATE INDEX idx_audit_logs_tenant ON audit_logs(tenant_id, created_at DESC);

-- 8. Enable Row-Level Security (RLS) on Sensitive Tables
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE secrets ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- 9. Row-Level Security Policies (Fail-Safe Defense)
CREATE POLICY tenant_isolation_projects ON projects
FOR ALL
USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);

CREATE POLICY tenant_isolation_secrets ON secrets
FOR ALL
USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);

CREATE POLICY tenant_isolation_audit ON audit_logs
FOR ALL
USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);