import type {
  Context,
  DeltaState,
  Feature,
  State,
  Strategy,
  StrategyDefinition
} from './types'

type MappedStrategy = {
  resultName: string
  strategyName: string
  strategy: Strategy
  parameters: Record<string, string>
  isEnabled: (context: Context) => boolean
}

type MappedFeature = {
  name: string
  strategies: MappedStrategy[]
}

export class Strategies {
  private knownStrategies: string[]
  private strategies: Map<string, Strategy> = new Map()
  private mappedFeatures: Map<string, MappedFeature> = new Map()

  constructor(knownStrategies: string[] = []) {
    this.knownStrategies = knownStrategies
  }

  private isCustomStrategy(strategyName: string): boolean {
    return !this.knownStrategies.includes(strategyName)
  }

  private mapCustomStrategies(
    strategies?: StrategyDefinition[]
  ): MappedStrategy[] {
    if (!strategies) {
      return []
    }

    return strategies
      .filter(
        ({ name }) => this.strategies.has(name) && this.isCustomStrategy(name)
      )
      .map((definition, index) => {
        const strategy = this.strategies.get(definition.name)!
        const parameters = definition.parameters ?? {}

        return {
          resultName: `customStrategy${index + 1}`,
          strategyName: definition.name,
          parameters,
          strategy,
          isEnabled: (context: Context) =>
            strategy.isEnabled(parameters, context)
        }
      })
  }

  private mapFeature(feature: Feature): MappedFeature {
    return {
      name: feature.name,
      strategies: this.mapCustomStrategies(feature.strategies)
    }
  }

  mapFeatures(state: State | DeltaState): void {
    if ('features' in state) {
      this.mappedFeatures = new Map(
        state.features.map(feature => [feature.name, this.mapFeature(feature)])
      )
      return
    }

    if ('events' in state) {
      state.events
        .sort((a, b) => a.eventId - b.eventId)
        .forEach(event => {
          if (event.type === 'hydration') {
            event.features.forEach(feature =>
              this.mappedFeatures.set(feature.name, this.mapFeature(feature))
            )
          }

          if (event.type === 'feature-updated') {
            this.mappedFeatures.set(
              event.feature.name,
              this.mapFeature(event.feature)
            )
          }

          if (event.type === 'feature-removed') {
            this.mappedFeatures.delete(event.featureName)
          }
        })
    }
  }

  registerCustomStrategies(strategies: Strategy[]): void {
    this.strategies = new Map(
      strategies.map(strategy => [strategy.name, strategy])
    )
  }

  getCustomStrategyPayload(toggleName: string, context: Context) {
    const feature = this.mappedFeatures.get(toggleName)
    if (!feature) {
      return {}
    }

    const strategies = Object.fromEntries(
      feature.strategies.map(strategy => [
        strategy.resultName,
        strategy.isEnabled(context)
      ])
    )

    return strategies
  }
}
