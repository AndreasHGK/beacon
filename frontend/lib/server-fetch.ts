import { cookies } from "next/headers"

export async function serverFetch(
  url: RequestInfo | string,
  options?: RequestInit
) {
  let finalUrl = url.toString()
  if (finalUrl[0] != "/") {
    throw new Error("`serverFetch` request must be sent to same site")
  }
  finalUrl = process.env.API_REQUEST_ROOT + finalUrl

  const finalOptions = options || ({} as { [key: string]: string })
  finalOptions["credentials"] = finalOptions["credentials"] || "include"
  finalOptions["headers"] = (finalOptions["headers"] || {}) as {
    [key: string]: string
  }
  // Make sure to pass along all cookies.
  finalOptions.headers["Cookie"] = cookies().toString()

  return await fetch(finalUrl, finalOptions)
}
