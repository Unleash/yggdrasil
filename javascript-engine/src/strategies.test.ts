import { describe, test, expect } from 'bun:test'
import { UnleashEngine, type State, type Strategy } from '.'

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
