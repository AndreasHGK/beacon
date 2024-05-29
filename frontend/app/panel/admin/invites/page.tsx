import { Dashboard } from "@/components/dashboard"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { CreateInviteForm } from "@/components/create-invite-form"

export default async function Users() {
  return (
    <Dashboard.Page>
      <Dashboard.Header>
        <Dashboard.Title>Invites</Dashboard.Title>
        <Dashboard.Subtext>
          Invites allows you to control which users can join your instance.
        </Dashboard.Subtext>
      </Dashboard.Header>
      <Dialog>
        <DialogTrigger asChild>
          <Button className="max-w-fit">Create invite code</Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create invite code</DialogTitle>
            <DialogDescription>
              Create an invite code which can be used to create an account.
              <br />
              <strong> Warning: </strong> everyone who sees this invite code can
              create an account.
            </DialogDescription>
          </DialogHeader>
          <CreateInviteForm />
        </DialogContent>
      </Dialog>
    </Dashboard.Page>
  )
}
