"use client"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { toast } from "sonner"
import { useRouter } from "next/navigation"
import { useCallback } from "react"

export function SSHKey(props: {
  owner_id: string
  name: string
  fingerprint: string
  add_date: Date
  last_use?: Date
}) {
  const router = useRouter()

  const deleteKey = useCallback(async () => {
    const resp = await fetch(
      `/api/users/${props.owner_id}/ssh-keys/${props.fingerprint.replaceAll("/", "%2F")}`,
      {
        method: "DELETE",
      }
    )

    if (resp.status === 401) {
      toast("You don't have permission to perform this action.")
      return
    }

    if (!resp.ok) {
      throw new Error("Could not delete SSH key.")
    }

    toast("The SSH key was removed.")
    router.refresh()
  }, [router, props.fingerprint, props.owner_id])

  return (
    <div className="border rounded p-2 px-3 flex flex-row bg-card">
      <div className="truncate shrink grow">
        <p className="text-lg font-semibold">{props.name}</p>
        <p className="text-sm py-1 text-ellipsis">{props.fingerprint}</p>
        <p className="text-xs pb-1">
          Added on{" "}
          {props.add_date.toLocaleString("en-GB", {
            year: "numeric",
            month: "long",
            day: "numeric",
          })}
          {" • "}
          {props.last_use
            ? "Last used on " +
              props.last_use.toLocaleString("en-GB", {
                year: "numeric",
                month: "long",
                day: "numeric",
                hour: "numeric",
                minute: "numeric",
                hour12: false,
              })
            : "Never used"}
        </p>
      </div>
      <div className="shrink-0 flex place-content-end pl-3">
        <Dialog>
          <DialogTrigger asChild>
            <Button className="my-auto" variant={"destructive"}>
              Delete
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Are you sure?</DialogTitle>
              <DialogDescription>
                You will no longer be able to use this key unless you re-add it.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant={"destructive"} onClick={deleteKey}>
                Confirm
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    </div>
  )
}
