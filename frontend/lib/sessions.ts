import { cookies } from "next/headers"
import { redirect } from "next/navigation"

// Returns true if the user has a session. This is a server-side function.
export function hasSession(): boolean {
  return cookies().has("session-token")
}

type SessionInfo = {
  token: string
  uuid: string
}

// Get information about the user's session, if there is one.
export function getSession(): SessionInfo | null {
  let token = cookies().get("session-token")?.value
  let uuid = cookies().get("session-uuid")?.value

  if (!token || !uuid) {
    return null
  }

  return {
    token: token,
    uuid: uuid,
  }
}

// Get the user's session, redirecting to the login page if there is no session present.
export function mustGetSession(): SessionInfo {
  const session = getSession()
  if (!session) {
    redirect("/login")
  }

  return session
}
