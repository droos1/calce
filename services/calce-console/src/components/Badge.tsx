import type { ReactNode } from 'react'

interface BadgeProps {
  variant?: 'neutral' | 'success' | 'warning' | 'error' | 'info'
  children: ReactNode
  className?: string
}

function Badge({ variant = 'neutral', children, className }: BadgeProps) {
  const classes = ['ds-badge', `ds-badge--${variant}`, className].filter(Boolean).join(' ')
  return <span className={classes}>{children}</span>
}

export default Badge
