import { useEffect, useState } from "react";
import { IconMoon, IconSun } from "./icons";

const STORAGE_KEY = "calce-theme";

function ThemeToggle() {
  const [theme, setTheme] = useState<"light" | "dark">(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored === "dark" ? "dark" : "light";
  });

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem(STORAGE_KEY, theme);
  }, [theme]);

  function toggle() {
    setTheme((prev) => (prev === "light" ? "dark" : "light"));
  }

  return (
    <button className="ds-btn ds-btn--ghost ds-btn--icon" onClick={toggle}>
      {theme === "light" ? <IconMoon /> : <IconSun />}
    </button>
  );
}

export default ThemeToggle;
