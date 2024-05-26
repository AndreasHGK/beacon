import { Dashboard } from "@/components/dashboard"

export default function Panel() {
  return (
    <Dashboard.Page>
      <Dashboard.Header>
        <Dashboard.Title>Overview</Dashboard.Title>
        <Dashboard.Subtext>Get a summary of your account.</Dashboard.Subtext>
      </Dashboard.Header>
    </Dashboard.Page>
  )
}
