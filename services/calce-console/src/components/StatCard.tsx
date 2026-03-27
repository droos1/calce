import { Link } from 'react-router'

interface StatCardProps {
  label: string
  value: string | number
  change?: string
  changeDirection?: 'positive' | 'negative'
  to?: string
}

function StatCard({ label, value, change, changeDirection, to }: StatCardProps) {
  const content = (
    <>
      <div className="ds-stat__label">{label}</div>
      <div className="ds-stat__value">{value}</div>
      {change && (
        <div className={`ds-stat__change ds-stat__change--${changeDirection || 'positive'}`}>
          {change}
        </div>
      )}
    </>
  )

  if (to) {
    return (
      <Link to={to} className="ds-stat ds-stat--clickable">
        {content}
      </Link>
    )
  }

  return <div className="ds-stat">{content}</div>
}

export default StatCard
