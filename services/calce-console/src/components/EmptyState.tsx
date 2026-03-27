import type { ReactNode } from 'react'

interface EmptyStateProps {
  title: string
  description?: string
  action?: ReactNode
}

function EmptyState({ title, description, action }: EmptyStateProps) {
  return (
    <div className="ds-empty">
      <div className="ds-empty__title">{title}</div>
      {description && <div className="ds-empty__description">{description}</div>}
      {action && <div className="ds-empty__action">{action}</div>}
    </div>
  )
}

export default EmptyState
