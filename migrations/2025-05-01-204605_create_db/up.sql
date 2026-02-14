-- Your SQL goes here
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT  NOT NULL,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP  NOT NULL 
);

INSERT INTO users (id, username, email, password_hash) 
values (0,"Not Assigned","nobody@nowhere.com", "" );
insert into users (username, password_hash, email) values ('admin','$2b$12$8/CLAWBT6MsKxkgptWfi6e4GpbZtdpJOyYCxbzfz6.Vy6oUn/O/tm','jschappet@gmail.com');

CREATE TABLE events (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP NOT NULL,
    location TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP  NOT NULL
);

INSERT INTO events (id, name, description, start_time, end_time, location)
VALUES ('1', 'Sample Event', 'This is a sample event description.', '2025-05-01 10:00:00', '2025-05-01 12:00:00', 'Sample Location');
INSERT INTO events (id, name, description, start_time, end_time, location)
VALUES ('MAY4_AR', 'May 4th Gathering', 'AR Games Night', '2025-05-04 13:00:00', '2025-05-04 16:00:00', 'Mount Vernon');


CREATE TABLE ticket (
    id TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    event_id TEXT NOT NULL,
    checked_in TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP  NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
);


CREATE TABLE registration (
    id INTEGER PRIMARY KEY AUTOINCREMENT  NOT NULL,
    event_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    attend BOOLEAN NOT NULL,
    notification BOOLEAN NOT NULL,
    source TEXT,
    comments TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP  NOT NULL,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
 );

