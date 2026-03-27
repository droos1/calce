interface SpinnerProps {
  size?: 'sm' | 'md' | 'lg'
  center?: boolean
}

function Spinner({ size = 'md', center = false }: SpinnerProps) {
  const spinner = <div className={`ds-spinner ds-spinner--${size}`} />
  if (center) {
    return <div className="ds-center">{spinner}</div>
  }
  return spinner
}

export default Spinner
