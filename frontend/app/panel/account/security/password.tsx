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

export function Password() {
  const session = mustGetSession()

  return (
    <div className="flex flex-col gap-2">
      <h2 className="font-bold text-2xl">Password</h2>
      <p className="text-lg text-muted-foreground">
        Press the button below to change your password. This will also clear all
        your sessions.
      </p>
      <div className="pt-2">
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
      </div>
    </div>
  )
}
