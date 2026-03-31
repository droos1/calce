import type { ReactNode } from 'react'

interface CardProps {
  header?: ReactNode
  actions?: ReactNode
  children: ReactNode
  className?: string
}

function Card({ header, actions, children, className }: CardProps) {
  return (
    <div className={['ds-card', className].filter(Boolean).join(' ')}>
      {(header || actions) && (
        <div className="ds-card__header">
          {header}
          {actions && <div className="ds-card__actions">{actions}</div>}
        </div>
      )}
      <div className="ds-card__body">{children}</div>
    </div>
  )
}

export default Card
