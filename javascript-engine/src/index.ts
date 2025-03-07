import init, { Engine } from '@unleash/yggdrasil-wasm'
import { checkResponse, readResponse } from './wasm-interop'
import { Strategies } from './strategies'
import type {
  Context,
  FeatureDefinition,
  MetricsBucket,
  State,
  Variant,
  Strategy
} from './types'
export * from './types'

await init()

export const DISABLED_VARIANT: Variant = {
  name: 'disabled',
  enabled: false,
  featureEnabled: false
}

export class UnleashEngine {
  private engine: Engine
  private strategies: Strategies

  constructor(strategies?: Strategy[]) {
    this.engine = new Engine()

    const knownStrategiesResponse = this.engine.builtInStrategies()
    const knownStrategies = readResponse<string[]>(knownStrategiesResponse)

    this.strategies = new Strategies(knownStrategies)

    if (strategies != null) {
      this.strategies.registerCustomStrategies(strategies)
    }
  }

  public takeState(state: State): void {
    const response = this.engine.takeState(state)
    checkResponse(response)

    this.strategies.mapFeatures(state)
  }

  public isEnabled(toggleName: string, context: Context): boolean | undefined {
    const customStrategyPayload = this.strategies.getCustomStrategyPayload(
      toggleName,
      context
    )

    const response = this.engine.checkEnabled(
      toggleName,
      context,
      customStrategyPayload
    )

    return readResponse<boolean>(response)
  }

  public getVariant(toggleName: string, context: Context): Variant | undefined {
    const customStrategyPayload = this.strategies.getCustomStrategyPayload(
      toggleName,
      context
    )

    const response = this.engine.checkVariant(
      toggleName,
      context,
      customStrategyPayload
    )

    return readResponse<Variant>(response)
  }

  public getMetrics(): MetricsBucket | undefined {
    const response = this.engine.getMetrics()
    return readResponse<MetricsBucket>(response)
  }

  public countFeature(featureName: string, enabled: boolean): void {
    const response = this.engine.countToggle(featureName, enabled)
    checkResponse(response)
  }

  public countVariant(featureName: string, variantName: string): void {
    const response = this.engine.countVariant(featureName, variantName)
    checkResponse(response)
  }

  public listKnownFeatures(): FeatureDefinition[] {
    const response = this.engine.listKnownFeatures()
    return readResponse<FeatureDefinition[]>(response) || []
  }
}
