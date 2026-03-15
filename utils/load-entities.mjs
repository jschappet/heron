// load-entities.js

import fs from "fs";
import { parse } from "csv-parse/sync";
import { fetchWithCookies, login } from "./loader-base.mjs";

const BASE_URL = "https://regenerateskagit.org";
const LEDGER_URL = `${BASE_URL}/api/ledger/submit/entities/bulk`;

const EMAIL = process.env.DEPLOY_EMAIL;
const PASSWORD = process.env.DEPLOY_PASSWORD;

async function loadCSV(file) {
  const raw = fs.readFileSync(file, "utf-8");

  return parse(raw, {
    columns: true,
    skip_empty_lines: true
  });
}

function transform(rows) {
  return rows.map(row => ({
    name: row.name,
    entity_type: row.entity_type,
    details: { imported: "Initial Import" }
  }));
}

async function postEntities(entities) {
  const res = await fetchWithCookies(LEDGER_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ rows: entities })
  });

  return res.json();
}

// ---- CLI ARG ----

const file = process.argv[2];

if (!file) {
  console.error("Usage: node load-entities.js <csv-file>");
  process.exit(1);
}

async function run() {
  console.log("Loading CSV");

  const rows = await loadCSV(file);
  const entities = transform(rows);

  console.log(`Posting ${entities.length} entities`);

  await login(BASE_URL, EMAIL, PASSWORD);

  const result = await postEntities(entities);

  console.log("✓ Import complete", result);
}

run().catch(err => {
  console.error(err);
  process.exit(1);
});