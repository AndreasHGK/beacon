"use client"

import { z } from "zod"
import { Button } from "@/components/ui/button"
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import { DialogFooter } from "@/components/ui/dialog"
import { Textarea } from "@/components/ui/textarea"
import { toast } from "sonner"
import { useRouter } from "next/navigation"

const formSchema = z.object({
  name: z.string().min(2).max(50),
  public_key: z.string(),
})

export function AddSSHKey(props: { userId: string }) {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })

  const router = useRouter()

  async function onSubmit(values: z.infer<typeof formSchema>) {
    const resp = await fetch(`/api/users/${props.userId}/ssh-keys`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(values),
    })

    if (resp.status == 422) {
      toast("Please enter a valid SSH key.")
      return
    }

    if (resp.status == 409) {
      toast("You have already added this key.")
      return
    }

    if (!resp.ok) {
      toast("Something went wrong.")
    } else {
      toast("You have added an SSH key to your account.")
    }

    router.refresh()
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <FormField
          control={form.control}
          name="name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Name</FormLabel>
              <FormControl>
                <Input placeholder="user@machine" {...field} />
              </FormControl>
              <FormDescription>
                This will be displayed to you in your list of SSH keys.
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="public_key"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Public SSH key</FormLabel>
              <FormControl>
                <Textarea placeholder="id-algorithm abcd" {...field} />
              </FormControl>
              <FormDescription>
                Your public SSH key. This is commonly a{" "}
                <span className="font-mono">.pub</span> file in the{" "}
                <span className="font-mono text-balance">~/.ssh/</span>{" "}
                directory.
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <DialogFooter>
          <Button type="submit">Submit</Button>
        </DialogFooter>
      </form>
    </Form>
  )
}
