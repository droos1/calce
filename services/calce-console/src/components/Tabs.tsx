interface TabsProps {
  tabs: string[]
  active: string
  onChange: (tab: string) => void
}

function Tabs({ tabs, active, onChange }: TabsProps) {
  return (
    <div className="ds-tabs">
      {tabs.map((tab) => (
        <button
          key={tab}
          className={`ds-tabs__tab${tab === active ? ' ds-tabs__tab--active' : ''}`}
          onClick={() => onChange(tab)}
        >
          {tab}
        </button>
      ))}
    </div>
  )
}

export default Tabs
