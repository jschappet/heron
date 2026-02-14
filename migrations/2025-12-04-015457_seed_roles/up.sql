-- Your SQL goes here
INSERT INTO roles (name, description, show_in_directory)
VALUES 
  ('member', 'Standard Revillage Society member', 1),
  ('admin', 'System administrator', 0),
  ('organizer', 'Community organizer', 1),
  ('volunteer', 'Community volunteer', 1),
  ('guest', 'Guest user with limited access', 0);
