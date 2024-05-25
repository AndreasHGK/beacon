"use client"

import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import { z } from "zod"
import { toast } from "sonner"
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
    password: z
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
  .refine((data) => data.password === data.verifyPassword, {
    message: "Passwords did not match",
    path: ["verifyPassword"],
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

type RegisterState =
  | { type: "idle" }
  | { type: "submitting" }
  | { type: "error"; message: string }

export function RegisterForm() {
  const router = useRouter()
  const [registerState, setRegisterState] = useState<RegisterState>({
    type: "idle",
  })

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {},
  })

  async function onSubmit(values: z.infer<typeof formSchema>) {
    setRegisterState({ type: "submitting" })

    let resp = await fetch("/api/users", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(values),
    })

    if (resp.status == 403) {
      setRegisterState({
        type: "error",
        message: await resp.text(),
      })
      return
    }

    if (!resp.ok) {
      setRegisterState({ type: "error", message: "An unknown error occurred." })
      throw new Error("An error occurred while trying to register")
    }

    setRegisterState({ type: "idle" })
    toast("Your account has been created.")
    router.replace("/")
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
              <FormLabel>Username</FormLabel>
              <FormControl>
                <Input placeholder="your username" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Password</FormLabel>
              <FormControl>
                <Input type="password" placeholder="your password" {...field} />
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
              <FormLabel>Confirm Password</FormLabel>
              <FormControl>
                <Input type="password" placeholder="your password" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        {(() => {
          // Display an error message if the login failed.
          if (registerState.type == "error") {
            return (
              <p className="text-sm font-medium text-destructive">
                {registerState.message}
              </p>
            )
          }
        })()}
        {(() => {
          // Display a loading button when the form is being processed.
          if (registerState.type == "idle" || registerState.type == "error") {
            return <Button type="submit">Submit</Button>
          } else if (registerState.type == "submitting") {
            return (
              <Button type="submit" disabled className="flex flex-row gap-2">
                <LoaderCircle className="animate-spin" />
                Submitting
              </Button>
            )
          }
        })()}
      </form>
    </Form>
  )
}
