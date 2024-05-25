import { serverFetch } from "./server-fetch"

type Config = {
  allow_registering: boolean
}

export async function getConfig(): Promise<Config> {
  let resp = await serverFetch("/api/config")
  if (!resp.ok) {
    throw new Error("Unable to fetch config")
  }

  return await resp.json()
}
