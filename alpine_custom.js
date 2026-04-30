let ROUTES = null;

/* =========================
Capability Loader
========================= */

async function loadRoutes() {
  if (ROUTES) return ROUTES;

  const res = await fetch("/api/config/capabilities");

  if (!res.ok) {
    throw new Error("Failed to load API capabilities");
  }

  const list = await res.json();

  // convert array → map keyed by capability key
  ROUTES = Object.fromEntries(
    list.map(r => [r.key, r])
  );

  console.log("Capabilities loaded:", ROUTES);

  return ROUTES;
}

/* =========================
URL Helpers
========================= */

function fillUrl(template, params = {}) {
  return template.replace(/{(\w+)}/g, (_, key) => {
    if (!(key in params)) {
      throw new Error(`Missing param: ${key}`);
    }
    return encodeURIComponent(params[key]);
  });
}

function buildQuery(params = {}) {
  const qs = new URLSearchParams();

  Object.entries(params).forEach(([k, v]) => {
    if (v !== undefined && v !== null && v !== "") {
      qs.append(k, v);
    }
  });

  return qs.toString();
}

/* =========================
Core API Caller
========================= */

async function apiCall(fnName, params = {}, body = null) {
  const routes = await loadRoutes();
  const route = routes[fnName];

  if (!route) {
    throw new Error(`Unknown API function: ${fnName}`);
  }

  let url = fillUrl(route.url, params);

  // prevent path params from also becoming query params
  if (route.method === "GET") {
    const pathKeys = [...route.url.matchAll(/{(\w+)}/g)].map(m => m[1]);


    const queryParams = Object.fromEntries(
      Object.entries(params).filter(([k]) => !pathKeys.includes(k))
    );

    const query = buildQuery(queryParams);
    if (query) url += `? ${query} `;


  }

  const headers = {
    "Content-Type": "application/json",
  };

  if (route.auth) {
    const token = localStorage.getItem("auth_token");
    if (token) headers["Authorization"] = `Bearer ${token}`;
  }

  const options = {
    method: route.method,
    headers,
  };

  if (body && route.method !== "GET") {
    options.body = JSON.stringify(body);
  }

  console.log("API CALL →", fnName);
  console.log("URL →", url);
  console.log("OPTIONS →", options);

  const res = await fetch(url, options);

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`API ${fnName} failed: ${res.status} ${text}`);
  }

  const ct = res.headers.get("content-type") || "";
  return ct.includes("application/json") ? res.json() : res.text();
}

/* =========================
Alpine Integration
========================= */

document.addEventListener("alpine:init", () => {

  Alpine.store("api", {
    async call(fnName, params = {}, body = null) {
      return apiCall(fnName, params, body);
    }
  });

  Alpine.store("auth", {
    loaded: false,
    loading: false,
    user: null,
    authenticated: false,
    roles: [],


    async load() {
      if (this.loaded || this.loading) return;

      this.loading = true;

      try {
        console.log("Loading profile…");

        const data = await Alpine.store("api").call("profile_json");

        this.user = data.user;
        this.authenticated = data.authenticated;
        this.roles = data.roles ?? [];

      } catch (e) {
        console.warn("Auth load failed:", e);
        this.user = null;
        this.authenticated = false;
        this.roles = [];
      } finally {
        this.loaded = true;
        this.loading = false;
      }
    }


  });

});

/* =========================
Preload capabilities early
========================= */

document.addEventListener("DOMContentLoaded", () => {
  loadRoutes().catch(err =>
    console.error("Failed to preload capabilities:", err)
  );
});

window.loadRoutes = loadRoutes;
