interface StatCardProps {
  label: string
  value: string | number
  change?: string
  changeDirection?: 'positive' | 'negative'
}

function StatCard({ label, value, change, changeDirection }: StatCardProps) {
  return (
    <div className="ds-stat">
      <div className="ds-stat__label">{label}</div>
      <div className="ds-stat__value">{value}</div>
      {change && (
        <div className={`ds-stat__change ds-stat__change--${changeDirection || 'positive'}`}>
          {change}
        </div>
      )}
    </div>
  )
}

export default StatCard
