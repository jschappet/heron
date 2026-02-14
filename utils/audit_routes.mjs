#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const ROOT = path.join(process.cwd(), "src/routes");
const MOD_FILE = path.join(ROOT, "mod.rs");
const OUTPUT_JS = path.join(process.cwd(), "./api-spec/api-routes.js");
const SCAN_JSON = path.join(process.cwd(), "./api-spec/route-scan.json");

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function mkDir() {
  const path = './api-spec';

  // Create directory asynchronously
  fs.mkdir(path, { recursive: true }, (err) => {
    if (err) {
      return console.error(err);
    }
    console.log('Directory created!');
  });
  
}
  

function parseRootScope() {
  const text = read(MOD_FILE);
  const m = text.match(/web::scope\("([^"]+)"\)/);
  return m ? m[1] : "";
}

function parseScopes() {
  const text = read(MOD_FILE);
  const scopeRegex = /scoped\("([^"]+)",\s*([\w:]+)::scope\(\)\)/g;
  const scopes = [];

  let match;
  while ((match = scopeRegex.exec(text))) {
    const basePath = match[1];
    const modulePath = match[2];
    const file = modulePath.split("::").pop() + ".rs";
    scopes.push({ basePath, file });
  }

  return scopes;
}

function parseRoutes(filePath, rootScope, basePath) {
  const text = read(filePath);
  const lines = text.split("\n");
  const results = [];
  const scanEntries = [];

  const attrRegex = /#\[(get|post|put|delete|patch)\("([^"]*)"\)\]/i;
  const ignoreRegex = /@audit-ignore/;

  // 1️⃣ Scan annotated functions
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(attrRegex);
    if (!m) continue;

    const httpMethod = m[1].toUpperCase();
    const subPath = m[2] || "/";

    for (let j = i + 1; j < i + 15 && j < lines.length; j++) {
      const fnMatch = lines[j].match(/async\s+fn\s+(\w+)/);
      if (!fnMatch) continue;

      const preLines = lines.slice(Math.max(0, j - 3), j).join("\n");
      if (ignoreRegex.test(preLines)) break;

      const fnName = fnMatch[1];
      const sigBlock = lines.slice(j, j + 20).join("\n");
      const usesAuth = sigBlock.includes("AuthContext");

      results.push({
        url: `${rootScope}${basePath}${subPath}`,
        fnName,
        httpMethod,
        location: `${path.relative(process.cwd(), filePath)}:${j + 1}`,
        usesAuth,
      });

      scanEntries.push({
        fnName,
        found: true,
        method: httpMethod,
        path: subPath,
        location: `${path.relative(process.cwd(), filePath)}:${j + 1}`,
      });

      break;
    }
  }

  // 2️⃣ Scan scope() for .service(fn_name)
  const scopeFnMatch = text.match(/pub\s+fn\s+scope\s*\(\)\s*->\s*Scope\s*{([\s\S]*?)}/);
  if (scopeFnMatch) {
    const scopeBody = scopeFnMatch[1];
    const serviceRegex = /\.service\((\w+)\)/g;
    let svcMatch;
    while ((svcMatch = serviceRegex.exec(scopeBody))) {
      const fnName = svcMatch[1];

      // Skip if already included
      if (scanEntries.find(r => r.fnName === fnName && r.found)) continue;

      // Find function declaration in file
      const fnLineIndex = lines.findIndex(line => new RegExp(`async\\s+fn\\s+${fnName}\\b`).test(line));
      if (fnLineIndex === -1) {
        scanEntries.push({
          fnName,
          found: false,
          method: null,
          path: null,
          location: `${path.relative(process.cwd(), filePath)}:??`,
        });
        continue;
      }

      const fnLine = lines[fnLineIndex];

      // ✅ Warn if function is public
      if (/pub\s+async\s+fn/.test(fnLine)) {
        console.warn(
          `⚠ Warning: scoped function '${fnName}' in ${path.relative(process.cwd(), filePath)}:${fnLineIndex + 1} is 'pub'. Consider removing 'pub' for encapsulation.`
        );
      }

      // Check @audit-ignore
      const preLines = lines.slice(Math.max(0, fnLineIndex - 3), fnLineIndex).join("\n");
      if (ignoreRegex.test(preLines)) continue;

      // Determine HTTP method and path from attribute if present
      let method = null;
      let subPath = null;
      for (let k = Math.max(0, fnLineIndex - 3); k <= fnLineIndex; k++) {
        const attrMatch = lines[k].match(attrRegex);
        if (attrMatch) {
          method = attrMatch[1].toUpperCase();
          subPath = attrMatch[2] || "/";
          break;
        }
      }

      const sigBlock = lines.slice(fnLineIndex, fnLineIndex + 20).join("\n");
      const usesAuth = sigBlock.includes("AuthContext");

      // Include in audit table only if we have a method/path
      if (method && subPath) {
        results.push({
          url: `${rootScope}${basePath}${subPath}`,
          fnName,
          httpMethod: method,
          location: `${path.relative(process.cwd(), filePath)}:${fnLineIndex + 1}`,
          usesAuth,
        });
      }

      scanEntries.push({
        fnName,
        found: !!method,
        method,
        path: subPath,
        location: `${path.relative(process.cwd(), filePath)}:${fnLineIndex + 1}`,
      });
    }
  }

  return { results, scanEntries };
}

function writeJsManifest(routes) {
  const obj = {};
  for (const r of routes) {
    obj[r.fnName] = {
      url: r.url,
      method: r.httpMethod,
      auth: r.usesAuth,
    };
  }

  const content =
`// AUTO-GENERATED — do not edit by hand
export const ROUTES = ${JSON.stringify(obj, null, 2)};
`;
  fs.writeFileSync(OUTPUT_JS, content);
  console.log(`Wrote JS manifest to ${OUTPUT_JS}`);
}


function writeScanFile(scanEntries) {
  fs.writeFileSync(SCAN_JSON, JSON.stringify(scanEntries, null, 2));
  console.log(`Wrote scan file to ${SCAN_JSON}`);
}

function audit() {
  const rootScope = parseRootScope();
  const scopes = parseScopes();
  const allResults = [];
  const allScanEntries = [];

  for (const s of scopes) {
    const filePath = path.join(ROOT, s.file);
    if (!fs.existsSync(filePath)) {
      console.warn(`Missing file: ${filePath}`);
      continue;
    }
    const { results, scanEntries } = parseRoutes(filePath, rootScope, s.basePath);
    allResults.push(...results);
    allScanEntries.push(...scanEntries);
  }

  console.log(`URL | Method Name | HTTP Method | "file:line" | Uses AuthContext`);
  console.log(`--- | ----------- | ----------- | ------------ | ---------------`);

  for (const r of allResults) {
    console.log(
      `${r.url} | ${r.fnName} | ${r.httpMethod} | "${r.location}" | ${r.usesAuth}`
    );
  }
  mkDir();

  writeJsManifest(allResults);
  writeScanFile(allScanEntries);
}

audit();
