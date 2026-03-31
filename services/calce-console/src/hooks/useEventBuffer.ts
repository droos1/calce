import { useRef, useState, useEffect, useCallback } from "react";

interface SseEvent {
  type: string;
  key: string;
  kind: string;
  time: string;
}

const MAX_RECENT = 50;
const FLUSH_INTERVAL_MS = 500;

export function useEventBuffer<C extends Record<string, number>>(
  initialCounts: C,
  classify: (event: SseEvent) => keyof C | null,
) {
  const [recentEvents, setRecentEvents] = useState<SseEvent[]>([]);
  const [totalEvents, setTotalEvents] = useState(0);
  const [counts, setCounts] = useState<C>(initialCounts);

  const pendingRef = useRef<SseEvent[]>([]);
  const pendingCountsRef = useRef<C>({ ...initialCounts });
  const classifyRef = useRef(classify);
  classifyRef.current = classify;

  const pushEvent = useCallback((raw: unknown) => {
    const event: SseEvent = { ...(raw as SseEvent), time: new Date().toISOString() };
    pendingRef.current.push(event);
    const key = classifyRef.current(event);
    if (key && key in pendingCountsRef.current) {
      (pendingCountsRef.current as Record<string, number>)[key as string]++;
    }
  }, []);

  const reset = useCallback(() => {
    pendingRef.current = [];
    pendingCountsRef.current = { ...initialCounts };
    setRecentEvents([]);
    setTotalEvents(0);
    setCounts(initialCounts);
  }, [initialCounts]);

  useEffect(() => {
    const interval = setInterval(() => {
      const pending = pendingRef.current;
      if (pending.length === 0) return;

      pendingRef.current = [];
      const flushed = pendingCountsRef.current;
      pendingCountsRef.current = { ...initialCounts };
      setRecentEvents((prev) => [...pending, ...prev].slice(0, MAX_RECENT));
      setTotalEvents((prev) => prev + pending.length);
      setCounts((prev) => {
        const next = { ...prev };
        for (const k of Object.keys(next)) {
          (next as Record<string, number>)[k] += (flushed as Record<string, number>)[k] ?? 0;
        }
        return next;
      });
    }, FLUSH_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [initialCounts]);

  return { recentEvents, totalEvents, counts, pushEvent, reset };
}
