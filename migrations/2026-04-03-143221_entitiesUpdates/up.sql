-- entity_types (global or per-host? keeping global for now)
-- Host Scoped for flexibility, but we can enforce global uniqueness on name
CREATE TABLE entity_types (
    id TEXT PRIMARY KEY NOT NULL,

    name TEXT NOT NULL UNIQUE,   -- 'person', 'project', 'organization', 'account'
    description TEXT NOT NULL,
    host_id INTEGER NOT NULL,

    can_hold_resources BOOLEAN DEFAULT 1 NOT NULL,
    can_initiate_flows BOOLEAN DEFAULT 1 NOT NULL,
    can_receive_flows BOOLEAN DEFAULT 1 NOT NULL
);

-- entity_identities (scoped to host)
CREATE TABLE entity_identities (
    id TEXT PRIMARY KEY NOT NULL,

    entity_id TEXT NOT NULL,
    host_id INTEGER NOT NULL,

    identity_type TEXT NOT NULL,  -- 'user_id', 'email', 'phone', 'external_id'
    identity_value TEXT NOT NULL,

    verified BOOLEAN DEFAULT 0 NOT NULL,
    primary_flag BOOLEAN DEFAULT 0 NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,

    FOREIGN KEY(entity_id)
        REFERENCES entities(id)
        ON DELETE CASCADE,

    FOREIGN KEY(host_id)
        REFERENCES hosts(id)
        ON DELETE CASCADE,

    UNIQUE(host_id, identity_type, identity_value, primary_flag)
);

CREATE INDEX idx_entity_identities_lookup
ON entity_identities(host_id, identity_type, identity_value);

CREATE UNIQUE INDEX idx_entity_identities_primary
ON entity_identities(entity_id, identity_type)
WHERE primary_flag = 1;

-- entity_relationships (scoped to host)
CREATE TABLE entity_relationships (
    id TEXT PRIMARY KEY NOT NULL,

    from_entity TEXT NOT NULL,
    to_entity TEXT NOT NULL,

    host_id INTEGER NOT NULL,

    relationship_type TEXT NOT NULL, -- 'member_of', 'owns', 'works_on', etc.

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,

    FOREIGN KEY(from_entity)
        REFERENCES entities(id)
        ON DELETE CASCADE,

    FOREIGN KEY(to_entity)
        REFERENCES entities(id)
        ON DELETE CASCADE,

    FOREIGN KEY(host_id)
        REFERENCES hosts(id)
        ON DELETE CASCADE
);


CREATE TABLE entity_merges (
    id TEXT PRIMARY KEY NOT NULL,
    from_entity TEXT NOT NULL,
    to_entity TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);


CREATE INDEX idx_entity_relationships_host
ON entity_relationships(host_id);

CREATE UNIQUE INDEX idx_entity_relationship_unique
ON entity_relationships(from_entity, to_entity, relationship_type);

-- extend entities
--ALTER TABLE entities ADD COLUMN entity_type_id TEXT;
--ALTER TABLE entities ADD COLUMN canonical_entity_id TEXT;

CREATE INDEX idx_entities_type_id ON entities(entity_type_id);