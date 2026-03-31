import { useRef, useState, useEffect, useCallback } from "react";
import { api } from "../api/client";

interface UseEventSourceOptions {
  onEvent: (data: unknown) => void;
  enabled?: boolean;
}

const MAX_BACKOFF_MS = 10_000;

export function useEventSource(url: string, options: UseEventSourceOptions) {
  const { onEvent, enabled } = options;
  const [connected, setConnected] = useState(false);
  const esRef = useRef<EventSource | null>(null);
  const backoffRef = useRef(1_000);
  const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;

  const cleanup = useCallback(() => {
    if (retryTimerRef.current) clearTimeout(retryTimerRef.current);
    if (esRef.current) {
      esRef.current.close();
      esRef.current = null;
    }
    setConnected(false);
  }, []);

  const connect = useCallback(() => {
    cleanup();

    const token = api.getAccessToken();
    if (!token) {
      console.warn("[sse] no token, cannot connect");
      return;
    }

    const fullUrl = `${url}?token=${encodeURIComponent(token)}`;
    const es = new EventSource(fullUrl);
    esRef.current = es;

    es.onopen = () => {
      console.info("[sse] connected to", url);
      setConnected(true);
      backoffRef.current = 1_000;
    };

    es.onerror = () => {
      // EventSource fires onerror for both transient reconnects and fatal closes.
      // If readyState is CLOSED the browser gave up — we need to reconnect manually.
      if (es.readyState === EventSource.CLOSED) {
        setConnected(false);
        esRef.current = null;
        const delay = backoffRef.current;
        backoffRef.current = Math.min(delay * 2, MAX_BACKOFF_MS);
        console.warn(`[sse] connection lost, reconnecting in ${delay}ms`);
        retryTimerRef.current = setTimeout(connect, delay);
      }
    };

    es.addEventListener("update", (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onEventRef.current(data);
      } catch (err) {
        console.warn("[sse] failed to parse event:", e.data, err);
      }
    });
  }, [url, cleanup]);

  const disconnect = useCallback(() => {
    cleanup();
    console.info("[sse] disconnected from", url);
  }, [url, cleanup]);

  // Auto-connect/disconnect when `enabled` changes
  useEffect(() => {
    if (enabled === undefined) return;
    if (enabled) {
      connect();
    } else {
      cleanup();
    }
  }, [enabled, connect, cleanup]);

  // Cleanup on unmount
  useEffect(() => cleanup, [cleanup]);

  return { connected, connect, disconnect };
}
