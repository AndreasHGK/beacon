import { hasSession } from "@/lib/sessions"
import { redirect } from "next/navigation"

export default function Home() {
  if (!hasSession()) {
    redirect("/login")
  } else {
    redirect("/panel")
  }
}
