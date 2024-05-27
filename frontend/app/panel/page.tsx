import { Dashboard } from "@/components/dashboard"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import { serverFetch } from "@/lib/server-fetch"
import { mustGetSession } from "@/lib/sessions"
import { redirect } from "next/navigation"
import prettyBytes from "pretty-bytes"
import { Suspense } from "react"

async function StatCards() {
  const session = mustGetSession()

  const resp = await serverFetch(`/api/users/${session.uuid}`)

  if (resp.status == 401) {
    redirect("/login")
  }
  if (!resp.ok) {
    throw new Error("could not get stats")
  }

  const data = (await resp.json()) as {
    total_storage_space: number
  }
  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>Total storage used</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-3xl font-semibold tracking-wide">
            {prettyBytes(data.total_storage_space)}
          </p>
        </CardContent>
      </Card>
    </>
  )
}

export default function Panel() {
  return (
    <Dashboard.Page>
      <Dashboard.Header>
        <Dashboard.Title>Overview</Dashboard.Title>
        <Dashboard.Subtext>Get a summary of your account.</Dashboard.Subtext>
      </Dashboard.Header>
      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
        <Suspense
          fallback={
            <>
              <Skeleton className="rounded-lg h-32" />
            </>
          }
        >
          <StatCards />
        </Suspense>
      </div>
    </Dashboard.Page>
  )
}
