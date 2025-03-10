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

export type Feature = {
  name: string
  enabled: boolean
  strategies?: StrategyDefinition[]
  variants?: Variant[]
}

enum Operator {
  IN = 'IN',
  NOT_IN = 'NOT_IN',
  STR_ENDS_WITH = 'STR_ENDS_WITH',
  STR_STARTS_WITH = 'STR_STARTS_WITH',
  STR_CONTAINS = 'STR_CONTAINS',
  NUM_EQ = 'NUM_EQ',
  NUM_GT = 'NUM_GT',
  NUM_GTE = 'NUM_GTE',
  NUM_LT = 'NUM_LT',
  NUM_LTE = 'NUM_LTE',
  DATE_AFTER = 'DATE_AFTER',
  DATE_BEFORE = 'DATE_BEFORE',
  SEMVER_EQ = 'SEMVER_EQ',
  SEMVER_GT = 'SEMVER_GT',
  SEMVER_LT = 'SEMVER_LT'
}

type Constraint = {
  contextName: string
  operator: Operator
  inverted: boolean
  values: string[]
  value?: string | number | Date
  caseInsensitive?: boolean
}

type Segment = {
  id: number
  constraints: Constraint[]
}

export type State = {
  version: number
  features: Feature[]
}

type DeltaHydrationEvent = {
  eventId: number
  type: 'hydration'
  segments: Segment[]
  features: Feature[]
}

type DeltaFeatureUpdatedEvent = {
  eventId: number
  type: 'feature-updated'
  feature: Feature
}

type DeltaFeatureRemovedEvent = {
  eventId: number
  type: 'feature-removed'
  featureName: string
  project: string
}

type DeltaEvent =
  | DeltaHydrationEvent
  | DeltaFeatureUpdatedEvent
  | DeltaFeatureRemovedEvent

export type DeltaState = {
  events: DeltaEvent[]
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
