"use client"

import { useRouter } from "next/navigation"
import { Button } from "./ui/button"
import { toast } from "sonner"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"

function LogoutConfirmButton() {
  const router = useRouter()

  const logout = async () => {
    const resp = await fetch("/api/logout", {
      method: "POST",
    })

    if (!resp.ok) {
      throw new Error("Could not log out")
    }

    toast("You have been logged out from your account")
    router.replace("/login")
  }

  return <Button onClick={logout}>Confirm</Button>
}

export function LogoutButton() {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button className="max-w-fit">Log out</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Are you sure?</DialogTitle>
          <DialogDescription>
            Are you sure you want to log out?
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <LogoutConfirmButton />
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
