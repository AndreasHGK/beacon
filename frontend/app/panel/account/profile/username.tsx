import { ChangeUsernameForm } from "@/components/change-username-form"
import { Dashboard } from "@/components/dashboard"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { serverFetch } from "@/lib/server-fetch"
import { mustGetSession } from "@/lib/sessions"

export default async function Username() {
  const session = mustGetSession()

  const resp = await serverFetch(`/api/users/${session.uuid}/username`)
  if (!resp.ok) {
    throw new Error("Failed to get username")
  }

  const username = await resp.json()

  return (
    <Dashboard.Section>
      <Dashboard.Subtitle>Username</Dashboard.Subtitle>
      <Dashboard.Subtext>
        Your username is <strong>{username}</strong>.
      </Dashboard.Subtext>
      <Dashboard.SectionContent>
        <Dialog>
          <DialogTrigger asChild>
            <Button>Change username</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Change username</DialogTitle>
              <DialogDescription>
                <strong>Notice: </strong>
                this change will also apply to the username you use to log in.
              </DialogDescription>
            </DialogHeader>
            <ChangeUsernameForm userId={session.uuid} />
          </DialogContent>
        </Dialog>
      </Dashboard.SectionContent>
    </Dashboard.Section>
  )
}
