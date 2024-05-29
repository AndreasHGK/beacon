"use client"

import { z } from "zod"
import { Button } from "@/components/ui/button"
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import { DialogFooter } from "@/components/ui/dialog"
import { toast } from "sonner"
import { useRouter } from "next/navigation"
import { useState } from "react"
import { LoaderCircle } from "lucide-react"

const formSchema = z
  .object({
    username: z
      .string()
      .min(3, {
        message: "Username must be at least 3 characters.",
      })
      .max(20, {
        message: "Username must be at most 20 characters.",
      }),
  })
  .refine(
    async (data) => {
      const resp = await fetch(`/api/usernames/${data.username}`)

      if (resp.status == 404) {
        // The username is available, continue.
        return true
      }
      if (!resp.ok) {
        throw new Error("failed to fetch username")
      }

      // The user was found, don't allow the signup.
      return false
    },
    {
      message: "Username not available",
      path: ["username"],
    }
  )

type FormState =
  | { type: "idle" }
  | { type: "submitting" }
  | { type: "error"; message: string }

export function ChangeUsernameForm(props: { userId: string }) {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })
  const [formState, setFormState] = useState<FormState>({
    type: "idle",
  })

  const router = useRouter()

  async function onSubmit(values: z.infer<typeof formSchema>) {
    setFormState({ type: "submitting" })

    const resp = await fetch(`/api/users/${props.userId}/username`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(values.username),
    })

    if (resp.status == 403) {
      setFormState({
        type: "error",
        message: "You are not authorized to perform this action.",
      })
      return
    }

    if (resp.status == 409) {
      setFormState({ type: "error", message: "That username is unavailable." })
      return
    }

    if (!resp.ok) {
      setFormState({ type: "error", message: "An unknown error occurred." })
      throw new Error("An error occurred while trying to change username")
    }

    setFormState({ type: "idle" })
    toast("Username has been changed successfully.")

    router.refresh()
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <FormField
          control={form.control}
          name="username"
          render={({ field }) => (
            <FormItem>
              <FormLabel>New Username</FormLabel>
              <FormControl>
                <Input
                  autoComplete="off"
                  spellCheck="false"
                  autoCapitalize="none"
                  placeholder="your new username"
                  {...field}
                />
              </FormControl>
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
