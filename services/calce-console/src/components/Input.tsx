import type { InputHTMLAttributes } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
}

function Input({ error = false, className, ...props }: InputProps) {
  const classes = ["ds-input", error && "ds-input--error", className]
    .filter(Boolean)
    .join(" ");

  return <input className={classes} {...props} />;
}

export default Input;
