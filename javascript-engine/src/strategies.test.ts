import { describe, test, expect } from 'bun:test'
import { UnleashEngine, type DeltaState, type State, type Strategy } from '.'

describe('Custom Strategy Tests', () => {
  test('Basic custom strategy', () => {
    const alwaysTrueStrategy: Strategy = {
      name: 'alwaysTrue',
      isEnabled: () => true
    }

    const engine = new UnleashEngine([alwaysTrueStrategy])
    const state: State = {
      version: 1,
      features: [
        {
          name: 'customFeature',
          enabled: true,
          strategies: [
            {
              name: 'alwaysTrue'
            }
          ]
        }
      ]
    }
    engine.takeState(state)

    expect(engine.isEnabled('customFeature', {})).toBe(true)
  })

  test('Multiple custom strategies', () => {
    const alwaysTrueStrategy: Strategy = {
      name: 'alwaysTrue',
      isEnabled: () => true
    }

    const alwaysFalseStrategy: Strategy = {
      name: 'alwaysFalse',
      isEnabled: () => false
    }

    const engine = new UnleashEngine([alwaysTrueStrategy, alwaysFalseStrategy])
    const state: State = {
      version: 1,
      features: [
        {
          name: 'multiStrategyFeature',
          enabled: true,
          strategies: [
            {
              name: 'alwaysTrue',
              parameters: {}
            },
            {
              name: 'alwaysFalse',
              parameters: {}
            }
          ]
        }
      ]
    }
    engine.takeState(state)

    expect(engine.isEnabled('multiStrategyFeature', {})).toBe(true)
  })

  test('Custom strategy with parameters', () => {
    const parameterizedStrategy: Strategy = {
      name: 'parameterized',
      isEnabled: (parameters, context) => {
        return parameters.requiredValue === context.properties?.testValue
      }
    }

    const engine = new UnleashEngine([parameterizedStrategy])
    const state: State = {
      version: 1,
      features: [
        {
          name: 'parameterizedFeature',
          enabled: true,
          strategies: [
            {
              name: 'parameterized',
              parameters: {
                requiredValue: 'test123'
              }
            }
          ]
        }
      ]
    }
    engine.takeState(state)

    expect(
      engine.isEnabled('parameterizedFeature', {
        properties: {
          testValue: 'test123'
        }
      })
    ).toBe(true)

    expect(
      engine.isEnabled('parameterizedFeature', {
        properties: {
          testValue: 'wrongValue'
        }
      })
    ).toBe(false)
  })

  test('Custom strategy with variants', () => {
    const variantStrategy: Strategy = {
      name: 'variantStrategy',
      isEnabled: () => true
    }

    const engine = new UnleashEngine([variantStrategy])
    const state: State = {
      version: 1,
      features: [
        {
          name: 'variantFeature',
          enabled: true,
          strategies: [
            {
              name: 'variantStrategy',
              parameters: {}
            }
          ],
          variants: [
            {
              name: 'testVariant',
              weight: 100,
              enabled: true,
              featureEnabled: true,
              payload: {
                type: 'string',
                value: 'test'
              }
            }
          ]
        }
      ]
    }
    engine.takeState(state)

    const variant = engine.getVariant('variantFeature', {})
    expect(variant?.name).toBe('testVariant')
    expect(variant?.enabled).toBe(true)
    expect(variant?.featureEnabled).toBe(true)
    expect(variant?.payload?.type).toBe('string')
    expect(variant?.payload?.value).toBe('test')
  })

  test('Built-in strategies are not overridden by custom strategies', () => {
    const userWithIdStrategy: Strategy = {
      name: 'userWithId',
      isEnabled: () => true
    }

    const engine = new UnleashEngine([userWithIdStrategy])
    const state: State = {
      version: 1,
      features: [
        {
          name: 'builtInFeature',
          enabled: true,
          strategies: [
            {
              name: 'userWithId',
              parameters: {
                userIds: 'user1,user2'
              }
            }
          ]
        }
      ]
    }
    engine.takeState(state)

    expect(engine.isEnabled('builtInFeature', { userId: 'user1' })).toBe(true)
    expect(engine.isEnabled('builtInFeature', { userId: 'user3' })).toBe(false)
  })
})

describe('Custom Strategy Tests with Delta Events', () => {
  test('Feature is updated through a delta event', () => {
    const l33tStrategy: Strategy = {
      name: 'l33tStrategy',
      isEnabled: (_, { userId }) => userId === '1337'
    }

    const engine = new UnleashEngine([l33tStrategy])
    const initialState: State = {
      version: 1,
      features: [
        {
          name: 'deltaFeature',
          enabled: false,
          strategies: [{ name: 'l33tStrategy' }]
        }
      ]
    }
    engine.takeState(initialState)

    expect(engine.isEnabled('deltaFeature', { userId: '42' })).toBe(false)
    expect(engine.isEnabled('deltaFeature', { userId: '1337' })).toBe(false)

    const delta: DeltaState = {
      events: [
        {
          eventId: 1,
          type: 'feature-updated',
          feature: {
            name: 'deltaFeature',
            enabled: true,
            strategies: [{ name: 'l33tStrategy' }]
          }
        }
      ]
    }
    engine.takeState(delta)

    expect(engine.isEnabled('deltaFeature', { userId: '42' })).toBe(false)
    expect(engine.isEnabled('deltaFeature', { userId: '1337' })).toBe(true)
  })

  test('Feature is removed through a delta event', () => {
    const alwaysTrueStrategy: Strategy = {
      name: 'alwaysTrue',
      isEnabled: () => true
    }

    const engine = new UnleashEngine([alwaysTrueStrategy])
    const initialState: State = {
      version: 1,
      features: [
        {
          name: 'featureToBeRemoved',
          enabled: true,
          strategies: [{ name: 'alwaysTrue' }]
        }
      ]
    }
    engine.takeState(initialState)

    expect(engine.isEnabled('featureToBeRemoved', {})).toBe(true)

    const delta: DeltaState = {
      events: [
        {
          eventId: 1,
          type: 'feature-removed',
          featureName: 'featureToBeRemoved',
          project: 'default'
        }
      ]
    }
    engine.takeState(delta)

    expect(engine.isEnabled('featureToBeRemoved', {})).toBeUndefined()
  })

  test('Hydration event loads multiple features', () => {
    const alwaysTrueStrategy: Strategy = {
      name: 'alwaysTrue',
      isEnabled: () => true
    }

    const engine = new UnleashEngine([alwaysTrueStrategy])

    const delta: DeltaState = {
      events: [
        {
          eventId: 1,
          type: 'hydration',
          segments: [],
          features: [
            {
              name: 'hydratedFeature1',
              enabled: true,
              strategies: [{ name: 'alwaysTrue' }]
            },
            {
              name: 'hydratedFeature2',
              enabled: false,
              strategies: [{ name: 'alwaysTrue' }]
            }
          ]
        }
      ]
    }
    engine.takeState(delta)

    expect(engine.isEnabled('hydratedFeature1', {})).toBe(true)
    expect(engine.isEnabled('hydratedFeature2', {})).toBe(false)
  })

  test('Retains known information and only latest delta update is applied', () => {
    const alwaysTrueStrategy: Strategy = {
      name: 'alwaysTrue',
      isEnabled: () => true
    }

    const engine = new UnleashEngine([alwaysTrueStrategy])

    const initialState: State = {
      version: 1,
      features: [
        {
          name: 'oldFeature',
          enabled: true,
          strategies: [{ name: 'alwaysTrue' }]
        },
        {
          name: 'feature1',
          enabled: false,
          strategies: [{ name: 'alwaysTrue' }]
        }
      ]
    }

    engine.takeState(initialState)

    expect(engine.isEnabled('oldFeature', {})).toBe(true)

    const delta: DeltaState = {
      events: [
        {
          eventId: 1,
          type: 'feature-updated',
          feature: {
            name: 'feature1',
            enabled: true,
            strategies: [{ name: 'alwaysTrue' }]
          }
        },
        {
          eventId: 2,
          type: 'feature-updated',
          feature: {
            name: 'feature2',
            enabled: false,
            strategies: [{ name: 'alwaysTrue' }]
          }
        },
        {
          eventId: 3,
          type: 'feature-updated',
          feature: {
            name: 'feature1',
            enabled: false,
            strategies: [{ name: 'alwaysTrue' }]
          }
        },
        {
          eventId: 4,
          type: 'feature-updated',
          feature: {
            name: 'feature2',
            enabled: true,
            strategies: [{ name: 'alwaysTrue' }]
          }
        }
      ]
    }

    engine.takeState(delta)

    expect(engine.isEnabled('oldFeature', {})).toBe(true)
    expect(engine.isEnabled('feature1', {})).toBe(false)
    expect(engine.isEnabled('feature2', {})).toBe(true)
  })
})
