import { describe, beforeEach, test, expect } from 'bun:test'
import { UnleashEngine, type Payload, type State, type Variant } from '.'

type BaseTest = {
  toggleName: string
  description: string
  context: Record<string, unknown>
}

type ToggleTest = BaseTest & {
  expectedResult: boolean
}

type VariantTest = BaseTest & {
  expectedResult: LegacyVariantResponse
}

type LegacyVariantResponse = Omit<Variant, 'featureEnabled'> & {
  feature_enabled: boolean
}

type TestSuite = {
  state: State
  tests: ToggleTest[]
  variantTests: VariantTest[]
}

describe('Client Spec Tests', () => {
  test('Client Spec', async () => {
    const basePath = '../client-specification/specifications'
    const indexFile = Bun.file(`${basePath}/index.json`)
    const testSuites = await indexFile.json()

    for (const suite of testSuites) {
      const suiteFile = Bun.file(`${basePath}/${suite}`)
      const {
        state,
        tests: toggleTests = [],
        variantTests = []
      }: TestSuite = await suiteFile.json()

      describe(`Suite: ${suite}`, () => {
        let engine: UnleashEngine

        beforeEach(() => {
          engine = new UnleashEngine()
          engine.takeState(state)
        })

        for (const toggleTest of toggleTests) {
          const { description, toggleName, context, expectedResult } =
            toggleTest

          test(`Toggle Test: ${description}`, () => {
            const toggleResponse = engine.isEnabled(toggleName, context)

            expect(toggleResponse).toBe(expectedResult)
          })
        }

        for (const variantTest of variantTests) {
          const toggleName = variantTest.toggleName
          const expectedResult = variantTest.expectedResult

          test(`Variant Test: ${variantTest.description}`, () => {
            const result = engine.getVariant(toggleName, variantTest.context)

            expect(result.name).toBe(expectedResult.name)
            expect(result.enabled).toBe(expectedResult.enabled)
            expect(result.featureEnabled).toBe(expectedResult.feature_enabled)
            expect<Payload | undefined>(result.payload).toEqual(
              expectedResult.payload
            )
          })
        }
      })
    }
  })
})
