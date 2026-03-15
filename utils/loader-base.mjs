// loader-base.js

import { CookieJar } from "tough-cookie";

const jar = new CookieJar();

export async function loadEntities(url) {

  const res = await fetchWithCookies(url, {});

  const data = await res.json();
  const entityMap = new Map(
    data.map(e => [e.name, e.id])
  );
  
  return entityMap;

  
}


export async function fetchWithCookies(url, options = {}) {
  const cookie = await jar.getCookieString(url);

  const res = await fetch(url, {
    ...options,
    headers: {
      ...(options.headers || {}),
      ...(cookie ? { cookie } : {})
    }
  });

  const setCookie = res.headers.get("set-cookie");
  if (setCookie) {
    await jar.setCookie(setCookie, url);
  }

  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`${res.status} ${res.statusText}\n${body}`);
  }

  return res;
}

export async function login(baseUrl, username, password) {
  await fetchWithCookies(`${baseUrl}/api/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      username,
      password
    })
  });
}