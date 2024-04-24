export async function serverFetch(
  url: RequestInfo | string,
  options?: RequestInit
) {
  let finalUrl = url.toString()
  if (finalUrl[0] != "/") {
    throw new Error("`serverFetch` request must be sent to same site")
  }
  finalUrl = process.env.API_REQUEST_ROOT + finalUrl

  const finalOptions = options || {}
  finalOptions["credentials"] = finalOptions["credentials"] || "include"

  return await fetch(finalUrl, finalOptions)
}
