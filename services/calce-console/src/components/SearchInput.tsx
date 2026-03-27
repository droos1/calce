import { IconSearch } from "./icons";

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

function SearchInput({ value, onChange, placeholder = "Search...", className }: SearchInputProps) {
  return (
    <div className={["ds-search", className].filter(Boolean).join(" ")}>
      <IconSearch className="ds-search__icon" />
      <input
        type="text"
        className="ds-search__input ds-input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
      />
    </div>
  );
}

export default SearchInput;
