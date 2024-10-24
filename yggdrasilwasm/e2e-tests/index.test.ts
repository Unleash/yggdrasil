import { describe, beforeEach, test, expect } from 'bun:test'
import { Engine } from '../pkg/yggdrasil_engine'

type BaseTest = {
  toggleName: string
  description: string
  context: Record<string, unknown>
}

type ToggleTest = BaseTest & {
  expectedResult: boolean
}

type VariantTest = BaseTest & {
  expectedResult: Record<string, unknown>
}

type VariantResponse = {
  featureEnabled: boolean,
  payload: Record<string, string>,
  enabled: boolean,
  name: string
}

type LegacyVariantResponse = {
  feature_enabled: boolean,
  payload: Record<string, string>,
  enabled: boolean,
  name: string
}

type TestSuite = {
  state: Record<string, unknown>
  tests: ToggleTest[]
  variantTests: VariantTest[]
}

const DISABLED_VARIANT = {
  name: 'disabled',
  enabled: false
}

const getDisabledVariant = (featureEnabled: boolean) => ({
  ...DISABLED_VARIANT,
  featureEnabled
})

type Response = {
  status_code: 'Ok' | 'Error' | 'NotFound'
  value: unknown | null
  error_message?: string
}

const extractResult = <T>(response: Response): T => {
  expect(response.error_message).toBeFalsy()
  expect(response.status_code).toBe('Ok')
  return response.value as T
}

describe('Client Spec Tests', () => {
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
      }: TestSuite = await suiteFile.json()

      describe(`Suite: ${suite}`, () => {
        let engine: Engine

        beforeEach(() => {
          engine = new Engine()
          engine.takeState(state)
        })

        for (const toggleTest of toggleTests) {
          const toggleName = toggleTest.toggleName
          const expectedResult = toggleTest.expectedResult

          test(`Toggle Test: ${toggleTest.description}`, () => {
            const toggleResponse = engine.checkEnabled(
              toggleName,
              toggleTest.context,
              undefined
            )

            const result = extractResult<boolean>(toggleResponse) ?? false

            expect(result).toBe(expectedResult)
          })
        }

        for (const variantTest of variantTests) {
          const toggleName = variantTest.toggleName
          const expectedResult = variantTest.expectedResult as any as LegacyVariantResponse;


          test(`Variant Test: ${variantTest.description}`, () => {
            const variantResponse = engine.checkVariant(
              toggleName,
              variantTest.context,
              undefined
            )

            const toggleResponse = engine.checkEnabled(
              toggleName,
              variantTest.context,
              undefined
            )

            const featureEnabled =
              extractResult<boolean>(toggleResponse) ?? false

            const result =
              extractResult<VariantResponse>(variantResponse) ??
              getDisabledVariant(featureEnabled)

            expect(result.name).toBe(expectedResult.name);
            expect(result.enabled).toBe(expectedResult.enabled);
            expect(result.featureEnabled).toBe(expectedResult.feature_enabled);
            expect(result.payload).toEqual(expectedResult.payload);
          })
        }
      })
    }
  })
})
