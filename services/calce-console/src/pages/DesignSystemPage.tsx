import Card from '../components/Card'

const spacing = [
  ['xs', '2px'],
  ['sm', '4px'],
  ['md', '8px'],
  ['lg', '12px'],
  ['xl', '16px'],
  ['2xl', '24px'],
  ['3xl', '32px'],
]

const colors = [
  ['Primary', '--color-primary'],
  ['Success', '--color-success'],
  ['Warning', '--color-warning'],
  ['Error', '--color-error'],
  ['Info', '--color-info'],
]

const surfaces = [
  ['Background', '--color-bg'],
  ['Surface', '--color-surface'],
  ['Surface Raised', '--color-surface-raised'],
  ['Surface Overlay', '--color-surface-overlay'],
]

const textColors = [
  ['Primary', '--color-text'],
  ['Secondary', '--color-text-secondary'],
  ['Tertiary', '--color-text-tertiary'],
]

const borders = [
  ['Border', '--color-border'],
  ['Border Light', '--color-border-light'],
  ['Border Focus', '--color-border-focus'],
]

function Swatch({ label, variable }: { label: string; variable: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-md)' }}>
      <div style={{ width: 32, height: 32, borderRadius: 'var(--radius-md)', background: `var(${variable})`, border: '1px solid var(--color-border-light)', flexShrink: 0 }} />
      <div>
        <div style={{ fontSize: 'var(--font-size-sm)', fontWeight: 500 }}>{label}</div>
        <div className="ds-text--mono" style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)' }}>{variable}</div>
      </div>
    </div>
  )
}

export default function DesignSystemPage() {
  return (
    <div className="ds-page">
      <div className="ds-page__header">
        <div>
          <h1 className="ds-page__title">System</h1>
          <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)', marginTop: 'var(--spacing-xs)' }}>
            Design tokens: colors, typography, and spacing
          </div>
        </div>
      </div>

      {/* Status colors */}
      <Card header="Status Colors">
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gap: 'var(--spacing-xl)' }}>
          {colors.map(([label, variable]) => (
            <Swatch key={variable} label={label} variable={variable} />
          ))}
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Surfaces */}
      <Card header="Surfaces">
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 'var(--spacing-xl)' }}>
          {surfaces.map(([label, variable]) => (
            <Swatch key={variable} label={label} variable={variable} />
          ))}
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Text Colors */}
      <Card header="Text Colors">
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 'var(--spacing-xl)' }}>
          {textColors.map(([label, variable]) => (
            <div key={variable} style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-md)' }}>
              <div style={{ fontSize: 'var(--font-size-sm)', fontWeight: 500, color: `var(${variable})` }}>{label}</div>
              <div className="ds-text--mono" style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)' }}>{variable}</div>
            </div>
          ))}
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Borders */}
      <Card header="Borders">
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 'var(--spacing-xl)' }}>
          {borders.map(([label, variable]) => (
            <Swatch key={variable} label={label} variable={variable} />
          ))}
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Typography */}
      <Card header="Typography">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-lg)' }}>
          <div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginBottom: 'var(--spacing-md)', textTransform: 'uppercase', letterSpacing: '0.04em', fontWeight: 600 }}>Sizes</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-sm)' }}>
              {(['2xl', 'xl', 'lg', 'base', 'sm', 'xs'] as const).map((size) => (
                <div key={size} style={{ display: 'flex', alignItems: 'baseline', gap: 'var(--spacing-xl)' }}>
                  <span style={{ width: 40, fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)' }}>{size}</span>
                  <span style={{ fontSize: `var(--font-size-${size})` }}>The quick brown fox</span>
                </div>
              ))}
            </div>
          </div>
          <div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginBottom: 'var(--spacing-md)', textTransform: 'uppercase', letterSpacing: '0.04em', fontWeight: 600 }}>Weights</div>
            <div style={{ display: 'flex', gap: 'var(--spacing-2xl)' }}>
              <span style={{ fontWeight: 400 }}>Regular (400)</span>
              <span style={{ fontWeight: 500 }}>Medium (500)</span>
              <span style={{ fontWeight: 600 }}>Semibold (600)</span>
              <span style={{ fontWeight: 700 }}>Bold (700)</span>
            </div>
          </div>
          <div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginBottom: 'var(--spacing-md)', textTransform: 'uppercase', letterSpacing: '0.04em', fontWeight: 600 }}>Monospace</div>
            <span className="ds-text--mono" style={{ fontSize: 'var(--font-size-base)' }}>
              const result = calculatePortfolioValue(holdings);
            </span>
          </div>
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Spacing */}
      <Card header="Spacing">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-md)' }}>
          {spacing.map(([name, value]) => (
            <div key={name} style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-xl)' }}>
              <span className="ds-text--mono" style={{ width: 40, fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)' }}>{name}</span>
              <div style={{ height: 12, width: `var(--spacing-${name})`, background: 'var(--color-primary)', borderRadius: 'var(--radius-sm)' }} />
              <span style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)' }}>{value}</span>
            </div>
          ))}
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Radii */}
      <Card header="Border Radius">
        <div style={{ display: 'flex', gap: 'var(--spacing-2xl)', alignItems: 'end' }}>
          {['sm', 'md', 'lg', 'xl', 'full'].map((r) => (
            <div key={r} style={{ textAlign: 'center' }}>
              <div style={{ width: 48, height: 48, borderRadius: `var(--radius-${r})`, background: 'var(--color-primary-light)', border: '2px solid var(--color-primary)' }} />
              <div className="ds-text--mono" style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginTop: 'var(--spacing-sm)' }}>{r}</div>
            </div>
          ))}
        </div>
      </Card>
    </div>
  )
}
