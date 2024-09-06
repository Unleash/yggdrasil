import { describe, beforeEach, test, expect } from 'bun:test'
import { Engine } from '../pkg/yggdrasil_engine'

describe('Client Spec Tests', () => {
  let engine: Engine

  beforeEach(() => {
    engine = new Engine()
  })

  test('Client Spec', async () => {
    const basePath = '../../client-specification/specifications'
    const indexFile = Bun.file(`${basePath}/index.json`)
    const testSuites = await indexFile.json()

    for (const suite of testSuites) {
      const suiteFile = Bun.file(`${basePath}/${suite}`)
      const {
        state,
        tests: toggleTests = [],
        variantTests = []
      } = await suiteFile.json()

      engine.takeState(state)

      describe(`Suite: ${suite}`, () => {
        for (const toggleTest of toggleTests) {
          const toggleName = toggleTest.toggleName as string
          const expectedResult = toggleTest.expectedResult as boolean

          test(`Toggle Test: ${toggleTest.description}`, () => {
            const result = engine.isEnabled(toggleName, toggleTest.context)
            expect(result).toBe(expectedResult)
          })
        }

        for (const variantTest of variantTests) {
          const toggleName = variantTest.toggleName as string
          const expectedResult = JSON.stringify(variantTest.expectedResult)

          test(`Variant Test: ${variantTest.description}`, () => {
            const result = engine.checkVariant(toggleName, variantTest.context)
            const jsonResult = JSON.stringify(result)
            expect(jsonResult).toBe(expectedResult)
          })
        }
      })
    }
  })
})
