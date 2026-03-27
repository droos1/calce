import type { ReactNode } from 'react'

interface CardProps {
  header?: ReactNode
  children: ReactNode
  className?: string
}

function Card({ header, children, className }: CardProps) {
  return (
    <div className={['ds-card', className].filter(Boolean).join(' ')}>
      {header && <div className="ds-card__header">{header}</div>}
      <div className="ds-card__body">{children}</div>
    </div>
  )
}

export default Card
