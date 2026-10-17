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

async function apiCall(fnName, params = {}, body = null, requestOptions = {}) {
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
    if (query) url += `?${query}`;


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
    credentials: "same-origin",
    signal: requestOptions.signal,
    cache: "no-store",
  };

  if (body && route.method !== "GET") {
    if (body instanceof URLSearchParams) {
      headers["Content-Type"] = "application/x-www-form-urlencoded";
      options.body = body.toString();
    } else if (typeof FormData !== "undefined" && body instanceof FormData) {
      delete headers["Content-Type"];
      options.body = body;
    } else {
      options.body = JSON.stringify(body);
    }
  }

  console.log("API CALL →", fnName);
  console.log("URL →", url);


  const res = await fetch(url, options);

  if (!res.ok) {
    const text = await res.text();
    const error = new Error(`API ${fnName} failed: ${res.status} ${text}`);
    error.status = res.status;
    throw error;
  }

  const ct = res.headers.get("content-type") || "";
  return ct.includes("application/json") ? res.json() : res.text();
}

/* =========================
Alpine Integration
========================= */

document.addEventListener("alpine:init", () => {

  Alpine.store("api", {
    async call(fnName, params = {}, body = null, options = {}) {
      return apiCall(fnName, params, body, options);
    }
  });

  Alpine.store("member", {

    inflows: [],
    outflows: [],
    entities: {},
    content: [],

    loading: false,

    async loadFlows() {

      const flows = await Alpine
        .store("api")
        .call("member_flows");

      this.inflows = flows.inflows ?? [];
      this.outflows = flows.outflows ?? [];

      this.entities = Object.fromEntries(
        (flows.entities ?? []).map(e => [e.id, e])
      );

    },

    async loadContent() {

      const data = await Alpine
        .store("api")
        .call("member_content");

      this.content = data.draft_list ?? [];

    },
    async loadAll() {

      if (this.loading || this.inflows.length || this.content.length) return;

      this.loading = true;

      await Promise.all([
        this.loadFlows(),
        this.loadContent()
      ]);

      this.loading = false;

    },

    get groupedContent() {

      return this.content.reduce((acc, item) => {

        acc[item.doc_type] ??= [];
        acc[item.doc_type].push(item);

        return acc;

      }, {});

    }

  });


  Alpine.store("profileEditor", {

    form: {},
    saving: false,
    message: "",

    start(user) {
      console.log("User: ", user);
      this.form = JSON.parse(JSON.stringify(user));
      this.message = "";

    },

    async save() {

      this.saving = true;

      try {

        const result = await Alpine
          .store("api")
          .call("update_member_profile", {}, {user_details: this.form});

        Alpine.store("auth").user = result.user;

        this.message = "Profile saved";

      } catch (e) {

        console.error(e);
        this.message = "Error saving profile";

      }

      this.saving = false;

    }

  });

  Alpine.store("auth", {
    loaded: false,
    loading: false,
    user: null,
    authenticated: false,
    roles: [],

    hasRole(roleName) {
      console.log("Checking role:", roleName, "in", this.roles);
      return this.roles?.includes(roleName);
    },

    async load() {
      console.log("Loading auth Profile...")
      if (this.loaded || this.loading) return;

      this.loading = true;

      try {
        console.log("Loading profile…");

        const data = await Alpine.store("api").call("profile_json");
        console.log("Data: ", data);

        this.user = data.user;
        this.authenticated = data.authenticated;
        this.roles = data.roles ?? [];
        //console.log("User: ", this.user);
      } catch (e) {
        //console.warn("Auth load failed:", e);
        this.user = null;
        this.authenticated = false;
        this.roles = [];
      } finally {
        this.loaded = true;
        this.loading = false;
      }
    }


  });
  let memberLoaded = false;

  Alpine.effect(() => {

    const auth = Alpine.store("auth");

    if (auth.loaded && auth.authenticated && !memberLoaded) {
      memberLoaded = true;
      Alpine.store("member").loadAll();
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
