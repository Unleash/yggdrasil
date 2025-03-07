export type Context = {
  userId?: string
  sessionId?: string
  remoteAddress?: string
  environment?: string
  appName?: string
  currentTime?: string
  properties?: Record<string, string>
}

export type Payload = {
  type: string
  value: string
}

export type Variant = {
  name: string
  payload?: Payload
  enabled: boolean
  featureEnabled: boolean
  weight?: number
}

export type StrategyDefinition = {
  name: string
  parameters?: Record<string, string>
}

type Feature = {
  name: string
  enabled: boolean
  strategies?: StrategyDefinition[]
  variants?: Variant[]
}

export type State = {
  version: number
  features: Feature[]
}

type FeatureCount = {
  yes: number
  no: number
  variants: Record<string, number>
}

export type MetricsBucket = {
  toggles: Record<string, FeatureCount>
  start: Date
  stop: Date
}

export type FeatureDefinition = {
  name: string
  project: string
  type?: string
}

export type Strategy = {
  name: string
  isEnabled: (parameters: Record<string, string>, context: Context) => boolean
}
