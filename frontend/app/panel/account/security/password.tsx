import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { ChangePasswordForm } from "@/components/change-password-form"
import { mustGetSession } from "@/lib/sessions"
import { Dashboard } from "@/components/dashboard"

export function Password() {
  const session = mustGetSession()

  return (
    <Dashboard.Section>
      <Dashboard.Subtitle>Password</Dashboard.Subtitle>
      <Dashboard.Subtext>
        Press the button below to change your password. This will also clear all
        your sessions.
      </Dashboard.Subtext>
      <Dashboard.SectionContent>
        <Dialog>
          <DialogTrigger asChild>
            <Button className="max-w-fit">Change password</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Change password</DialogTitle>
              <DialogDescription>
                Your previous password is required in order to change your
                password.
              </DialogDescription>
            </DialogHeader>
            <ChangePasswordForm userId={session.uuid} />
          </DialogContent>
        </Dialog>
      </Dashboard.SectionContent>
    </Dashboard.Section>
  )
}
