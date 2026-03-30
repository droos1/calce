(() => {
  "use strict";

  // ── State ──────────────────────────────────────────────────────────────
  let accessToken = null;
  let refreshToken = null;
  let expiresAt = 0;         // epoch ms
  let refreshTimer = null;
  let chatHistory = [];

  const $ = (sel) => document.querySelector(sel);
  const loginScreen = $("#login-screen");
  const chatScreen  = $("#chat-screen");
  const messages     = $("#messages");

  // ── Auth helpers ───────────────────────────────────────────────────────

  async function apiFetch(url, opts = {}) {
    const headers = { "Content-Type": "application/json", ...opts.headers };
    if (accessToken) headers["Authorization"] = `Bearer ${accessToken}`;
    const res = await fetch(url, { ...opts, headers });
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(body.detail || body.message || `HTTP ${res.status}`);
    }
    return res;
  }

  function scheduleRefresh(expiresIn) {
    clearTimeout(refreshTimer);
    // Refresh 60 s before expiry, minimum 10 s
    const ms = Math.max((expiresIn - 60) * 1000, 10_000);
    refreshTimer = setTimeout(doRefresh, ms);
  }

  async function doRefresh() {
    if (!refreshToken) return;
    try {
      const res = await apiFetch("/auth/refresh", {
        method: "POST",
        body: JSON.stringify({ refresh_token: refreshToken }),
      });
      const data = await res.json();
      accessToken  = data.access_token;
      refreshToken = data.refresh_token;
      expiresAt    = Date.now() + data.expires_in * 1000;
      scheduleRefresh(data.expires_in);
    } catch {
      doLogout();
    }
  }

  function doLogout() {
    if (refreshToken) {
      fetch("/auth/logout", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: refreshToken }),
      }).catch(() => {});
    }
    accessToken = null;
    refreshToken = null;
    expiresAt = 0;
    chatHistory = [];
    clearTimeout(refreshTimer);
    chatScreen.hidden  = true;
    loginScreen.hidden = false;
    messages.innerHTML = "";
    $("#email").value = "";
    $("#password").value = "";
    $("#login-error").hidden = true;
  }

  // ── Login ──────────────────────────────────────────────────────────────

  $("#login-form").addEventListener("submit", async (e) => {
    e.preventDefault();
    const btn   = $("#login-btn");
    const error = $("#login-error");
    error.hidden = true;
    btn.disabled = true;
    btn.textContent = "Logging in...";

    try {
      const res = await apiFetch("/auth/login", {
        method: "POST",
        body: JSON.stringify({
          email: $("#email").value,
          password: $("#password").value,
        }),
      });
      const data = await res.json();
      accessToken  = data.access_token;
      refreshToken = data.refresh_token;
      expiresAt    = Date.now() + data.expires_in * 1000;
      scheduleRefresh(data.expires_in);

      // Parse JWT payload for display
      const payload = JSON.parse(atob(accessToken.split(".")[1]));
      $("#user-info").textContent = `${payload.sub} (${payload.role})`;

      loginScreen.hidden = true;
      chatScreen.hidden  = false;
      $("#chat-input").focus();
    } catch (err) {
      error.textContent = err.message;
      error.hidden = false;
    } finally {
      btn.disabled = false;
      btn.textContent = "Log in";
    }
  });

  $("#logout-btn").addEventListener("click", doLogout);

  // ── Chat ───────────────────────────────────────────────────────────────

  function appendMessage(role, text) {
    const div = document.createElement("div");
    div.className = `msg ${role}`;
    div.textContent = text;
    messages.appendChild(div);
    messages.scrollTop = messages.scrollHeight;
    return div;
  }

  $("#chat-form").addEventListener("submit", async (e) => {
    e.preventDefault();
    const input = $("#chat-input");
    const text  = input.value.trim();
    if (!text) return;

    input.value = "";
    $("#send-btn").disabled = true;
    appendMessage("user", text);

    const assistantDiv = appendMessage("assistant", "");
    assistantDiv.classList.add("streaming");

    try {
      const res = await fetch("/chat", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${accessToken}`,
        },
        body: JSON.stringify({
          message: text,
          history: chatHistory,
        }),
      });

      if (!res.ok) {
        if (res.status === 401) {
          doLogout();
          return;
        }
        const body = await res.json().catch(() => ({}));
        throw new Error(body.detail || `HTTP ${res.status}`);
      }

      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      let fullText = "";

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop(); // keep incomplete line

        let eventType = "";
        for (const line of lines) {
          if (line.startsWith("event: ")) {
            eventType = line.slice(7).trim();
          } else if (line.startsWith("data: ")) {
            const raw = line.slice(6);
            try {
              const data = JSON.parse(raw);
              if (eventType === "text") {
                fullText += data.content;
                assistantDiv.textContent = fullText;
                messages.scrollTop = messages.scrollHeight;
              } else if (eventType === "tool_call") {
                const tag = document.createElement("span");
                tag.className = "tool-tag";
                tag.textContent = data.name;
                assistantDiv.appendChild(tag);
                assistantDiv.appendChild(document.createTextNode("\n"));
              } else if (eventType === "error") {
                fullText += `\n[Error: ${data.message}]`;
                assistantDiv.textContent = fullText;
              }
            } catch {
              // skip malformed JSON
            }
            eventType = "";
          }
        }
      }

      // Update history for multi-turn
      chatHistory.push({ role: "user", content: text });
      chatHistory.push({ role: "assistant", content: fullText });

    } catch (err) {
      assistantDiv.textContent = `Error: ${err.message}`;
    } finally {
      assistantDiv.classList.remove("streaming");
      $("#send-btn").disabled = false;
      input.focus();
    }
  });
})();
