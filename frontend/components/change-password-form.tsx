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
    sender_current_password: z.string().min(2).max(50),
    target_new_password: z
      .string()
      .min(8, {
        message: "Password must be at least 8 characters",
      })
      .max(80, {
        message: "Password must be at most 80 characters",
      }),
    verifyPassword: z
      .string()
      .min(8, {
        message: "Password must be at least 8 characters",
      })
      .max(80, {
        message: "Password must be at most 80 characters",
      }),
  })
  .refine((data) => data.target_new_password === data.verifyPassword, {
    message: "Passwords did not match",
    path: ["verifyPassword"],
  })

type FormState =
  | { type: "idle" }
  | { type: "submitting" }
  | { type: "error"; message: string }

export function ChangePasswordForm(props: { userId: string }) {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })
  const [formState, setFormState] = useState<FormState>({
    type: "idle",
  })

  const router = useRouter()

  async function onSubmit(values: z.infer<typeof formSchema>) {
    const resp = await fetch(`/api/users/${props.userId}/password`, {
      method: "PUT",
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

    if (resp.status == 401) {
      setFormState({ type: "error", message: "Credentials did not match." })
      return
    }

    if (!resp.ok) {
      setFormState({ type: "error", message: "An unknown error occurred." })
      throw new Error("An error occurred while trying to register")
    }

    setFormState({ type: "idle" })
    toast("User password has been changed successfully.")

    router.refresh()
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <FormField
          control={form.control}
          name="sender_current_password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Your Current Password</FormLabel>
              <FormControl>
                <Input
                  type="password"
                  autoComplete="current-password"
                  placeholder="current password"
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="target_new_password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>New Password</FormLabel>
              <FormControl>
                <Input
                  type="password"
                  autoComplete="new-password"
                  placeholder="new password"
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="verifyPassword"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Confirm New Password</FormLabel>
              <FormControl>
                <Input
                  type="password"
                  autoComplete="new-password"
                  placeholder="new password"
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        {(() => {
          // Display an error message if the login failed.
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
