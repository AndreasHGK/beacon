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
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import { DialogFooter } from "@/components/ui/dialog"
import { toast } from "sonner"
import { useState } from "react"
import { LoaderCircle } from "lucide-react"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Input } from "@/components/ui/input"

const formSchema = z.object({
  invite_code: z.string().min(8).max(64),
  valid_for: z.coerce.number(),
  max_uses: z.coerce.number().min(1).max(100),
})

type FormState =
  | { type: "idle" }
  | { type: "submitting" }
  | { type: "error"; message: string }

export function CreateInviteForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: { max_uses: 1 },
  })
  const [formState, setFormState] = useState<FormState>({
    type: "idle",
  })

  async function onSubmit(values: z.infer<typeof formSchema>) {
    setFormState({ type: "submitting" })

    const resp = await fetch(`/api/invites`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(values),
    })

    if (resp.status == 403) {
      setFormState({
        type: "error",
        message: "You are not authorized to perform this action.",
      })
      return
    }

    if (resp.status == 409) {
      setFormState({
        type: "error",
        message: "That invite code already exists.",
      })
      return
    }

    if (!resp.ok) {
      setFormState({ type: "error", message: "An unknown error occurred." })
      throw new Error("An error occurred while trying to create an invite")
    }

    navigator.clipboard.writeText(values.invite_code)

    setFormState({ type: "idle" })
    toast("Invite code has been created and copied to clipboard")
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <FormField
          control={form.control}
          name="invite_code"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Invite Code</FormLabel>
              <FormControl>
                <Input placeholder="a hard to guess code" {...field} />
              </FormControl>
              <FormDescription>
                The code people use to create an account. This should not be
                easy to guess to avoid unwanted visitors.
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="valid_for"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Expires After</FormLabel>
              <Select onValueChange={field.onChange}>
                <FormControl>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a duration" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  <SelectItem value={(60 * 30).toString()}>
                    30 minutes
                  </SelectItem>
                  <SelectItem value={(60 * 60 * 1).toString()}>
                    1 hour
                  </SelectItem>
                  <SelectItem value={(60 * 60 * 6).toString()}>
                    6 hours
                  </SelectItem>
                  <SelectItem value={(60 * 60 * 12).toString()}>
                    12 hours
                  </SelectItem>
                  <SelectItem value={(60 * 60 * 24 * 1).toString()}>
                    1 day
                  </SelectItem>
                  <SelectItem value={(60 * 60 * 24 * 3).toString()}>
                    3 days
                  </SelectItem>
                  <SelectItem value={(60 * 60 * 24 * 7).toString()}>
                    7 days
                  </SelectItem>
                </SelectContent>
              </Select>
              <FormDescription>
                After the provided time the invite will no longer be usable,
                regardless of if it was used or not.
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="max_uses"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Max Uses</FormLabel>
              <FormControl>
                <Input placeholder="a number" {...field} />
              </FormControl>
              <FormDescription>
                Controls the maximum amount of accounts that can be created with
                this code before it expires.
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        {(() => {
          if (formState.type == "error") {
            return (
              <p className="text-sm font-medium text-destructive">
                {formState.message}
              </p>
            )
          }
        })()}
        <DialogFooter>
          {(() => {
            // Display a loading button when the form is being processed.
            if (formState.type == "idle" || formState.type == "error") {
              return <Button type="submit">Submit</Button>
            } else if (formState.type == "submitting") {
              return (
                <Button type="submit" disabled className="flex flex-row gap-2">
                  <LoaderCircle className="animate-spin" />
                  Submitting
                </Button>
              )
            }
          })()}
        </DialogFooter>
      </form>
    </Form>
  )
}
