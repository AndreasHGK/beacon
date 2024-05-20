import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { serverFetch } from "@/lib/server-fetch"
import { mustGetSession } from "@/lib/sessions"
import { redirect } from "next/navigation"
import { AddSSHKey } from "./ssh-key-form"
import { SSHKey } from "@/components/ssh-key"

export async function SSHKeys() {
  const session = mustGetSession()

  const resp = await serverFetch(`/api/users/${session.uuid}/ssh-keys`)
  if (resp.status == 401) {
    redirect("/login")
  }
  if (resp.status != 200) {
    throw new Error("Failed to fetch ssh keys")
  }

  const sshKeys = (await resp.json()) as {
    name: string
    fingerprint: string
    add_date: number
    last_use?: number
  }[]

  return (
    <div className="flex flex-col gap-2">
      <h2 className="font-bold text-2xl">SSH Keys</h2>
      <p className="text-lg text-muted-foreground">
        To avoid needing to manually authenticate when using the CLI, you may
        provide public SSH keys to automatically authenticate via the terminal.
      </p>
      <div className="pt-2">
        <Dialog>
          <DialogTrigger asChild>
            <Button className="max-w-fit">Add key</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add an SSH key</DialogTitle>
              <DialogDescription>
                Please enter the <strong>public</strong> SSH key you would like
                to add to your account. Be careful! This will allow anyone that
                knows the corresponding private key to access your account.
              </DialogDescription>
            </DialogHeader>
            <AddSSHKey userId={session.uuid} />
          </DialogContent>
        </Dialog>
      </div>
      <div className="grid col-1 gap-4 pt-4">
        {sshKeys.map((val, id) => {
          return (
            <SSHKey
              key={id}
              owner_id={session.uuid}
              name={val.name}
              fingerprint={val.fingerprint}
              add_date={new Date(val.add_date)}
              last_use={val.last_use ? new Date(val.last_use) : undefined}
            />
          )
        })}
      </div>
    </div>
  )
}
