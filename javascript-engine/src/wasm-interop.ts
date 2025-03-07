type EngineResponse = {
  error_message?: string
  status_code?: string
}

type EngineResponseWithValue<T> = EngineResponse & {
  value?: T
}

export const readResponse = <T>(
  response: EngineResponseWithValue<T>
): T | undefined => {
  if (response.status_code === 'Error') {
    throw new Error(response.error_message || 'Unknown error')
  }

  return response.value
}

export const checkResponse = (response: EngineResponse) => {
  if (response.status_code === 'Error') {
    throw new Error(response.error_message || 'Unknown error')
  }
}
