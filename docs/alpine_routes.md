# AlpineJS + ROUTES Cheat Sheet

This document provides guidance on using the `ROUTES` API abstraction in AlpineJS components and modals.

---

## 1. What `ROUTES` Is

* A JS object mapping **function names** to backend endpoints, HTTP methods, and auth requirements.

```js
ROUTES = {
  login: { url: "/api/auth/login", method: "POST", auth: false },
  get_user_api: { url: "/api/users/{user_id}", method: "GET", auth: true },
};
```

* Frontend only uses function names (`login`, `get_user_api`), not hardcoded URLs.

---

## 2. Using ROUTES in AlpineJS

### Inside JS Methods (x-data)

Use `Alpine.store("api")` to call routes.

```js
async loadUser(userId) {
  const data = await Alpine.store("api").call("get_user_api", { user_id: userId });
  this.user = data;
}
```

### Inside Alpine Template Expressions

`$store` works in `x-text`, `@click`, `x-init`, etc.

```html
<button @click="$store.api.call('login', {}, { email, password })">Login</button>
```

---

## 3. URL Parameters

URLs can have placeholders like `{user_id}` or `{token}`.
Use a helper to fill them:

```js
function fillUrl(template, params = {}) {
  return template.replace(/{(\w+)}/g, (_, key) => {
    if (!(key in params)) throw new Error(`Missing param: ${key}`);
    return encodeURIComponent(params[key]);
  });
}
```

---

## 4. Auth Handling

* Routes with `auth: true` automatically include the stored token:

```js
if (route.auth) {
  const token = localStorage.getItem("auth_token");
  if (token) headers["Authorization"] = `Bearer ${token}`;
}
```

* Components never manage tokens directly.

---

## 5. Generic API Call Signature

```js
Alpine.store("api").call(fnName, params = {}, body = null)
```

* `fnName` → function name in `ROUTES`
* `params` → object for URL placeholders
* `body` → POST/PUT payload

**Example:**

```js
await Alpine.store("api").call("create_wants_to_contribute", {}, {
  offer_id: 42,
  who: "Alice",
  how_helping: "Delivery",
});
```

---

## 6. Error Handling

```js
try {
  const data = await Alpine.store("api").call("get_user_api", { user_id: 42 });
} catch (e) {
  console.error("API call failed:", e);
}
```

* Throws on unknown fnName, missing params, or non-2xx responses.

---

## 7. Integration Patterns

* **Modal forms** → bind fields with `x-model`, submit via `api.call()`
* **Page init** → fetch data in `x-init` using `Alpine.store("api").call()`
* **Loops / dynamic data** → fetch profiles, offers, etc., using route names

---

## 8. Common Gotchas

1. `$store` only works inside Alpine template expressions.
2. Ensure `api` store is registered before components access it.
3. `ROUTES` must be imported or global depending on your module setup.
4. Use `x-cloak` on modals to avoid flashes of unstyled content.

---

**Tip:** Always treat `ROUTES` as the **single source of truth** for backend paths. Frontend logic should never hardcode URLs or HTTP methods.
