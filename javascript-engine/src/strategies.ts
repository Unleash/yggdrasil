import type { Context, State, Strategy, StrategyDefinition } from './types'

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
  private mappedFeatures?: Map<string, MappedFeature>

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

  mapFeatures({ features }: State): void {
    if (!features) return

    this.mappedFeatures = new Map(
      features.map(feature => [
        feature.name,
        {
          name: feature.name,
          strategies: this.mapCustomStrategies(feature.strategies)
        }
      ])
    )
  }

  registerCustomStrategies(strategies: Strategy[]): void {
    this.strategies = new Map(
      strategies.map(strategy => [strategy.name, strategy])
    )
  }

  getCustomStrategyPayload(toggleName: string, context: Context) {
    const feature = this.mappedFeatures?.get(toggleName)
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
