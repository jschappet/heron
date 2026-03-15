// load-entities.js

import fs from "fs";
import { parse } from "csv-parse/sync";
import { fetchWithCookies, loadEntities, login } from "./loader-base.mjs";

const BASE_URL = "https://regenerateskagit.org";
const LEDGER_URL = `${BASE_URL}/api/ledger/submit/bulk`;
const ENTITIES_URL = `${BASE_URL}/api/ledger/entities`;

const EMAIL = process.env.DEPLOY_EMAIL;
const PASSWORD = process.env.DEPLOY_PASSWORD;
const resourceMap = {
  "donation":"USD",
"hosted" : "count",
"labor_time" : "hours",
"meal" : "count",
"member_of" : "count",
"presentation" : "count",
"tour" : "count",
"visited" : "count",
}
async function loadCSV(file) {
  const raw = fs.readFileSync(file, "utf-8");

  return parse(raw, {
    columns: true,
    skip_empty_lines: true
  });
}

function parseDate(date, input_time) {
  let [year, month, day] = date.split("-");
  if (year.length === 2) year = `20${year}`;
  month = month.padStart(2, "0");
  day = day.padStart(2, "0");
  const time = input_time?.trim() || "08:00:00";
  const isoString = `${year}-${month}-${day}T${time}`;
  const d = new Date(isoString);
  if (isNaN(d)) throw new Error(`Invalid date: ${isoString}`);
  return d.toISOString().slice(0, 19);

}

function parseToNumber(input) {
  const quantity = Number(input);

  if (!Number.isFinite(quantity) || quantity <= 0) {
    return null;
  }

  return quantity;
}
function transform(rows, entities) {
  return rows.map(row => ({
    from_entity: entities.get(row.from),
    to_entity: entities.get(row.to),
    quantity_value: parseToNumber(row.quantity),
    quantity_unit: resourceMap[row.resource_type],
    notes: row.notes,
    timestamp: parseDate(row.date, row.time),
    details: { imported: "Ledger loader" },
    resource_type: row.resource_type
    
  }));
}

async function postFlowEvents(entities) {
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

  await login(BASE_URL, EMAIL, PASSWORD);

  const entities = await loadEntities(ENTITIES_URL)

  //console.log(entities);
  
  const flow_events = transform(rows, entities);

  console.log(`Posting ${flow_events.length} Events`);
  console.log(flow_events);
  const result = await postFlowEvents(flow_events);

  console.log("✓ Import complete", result);
}

run().catch(err => {
  console.error(err);
  process.exit(1);
});