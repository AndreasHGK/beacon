import { ReactNode } from "react"

export const Dashboard = {
  Page: (props: { children: ReactNode }) => {
    return (
      <main className="flex flex-col justify-center flex-1 gap-8">
        {props.children}
      </main>
    )
  },

  Header: (props: { children: ReactNode }) => {
    return <div className="flex flex-col gap-2">{props.children}</div>
  },

  Section: (props: { children: ReactNode }) => {
    return <div className="flex flex-col gap-2">{props.children}</div>
  },

  SectionContent: (props: { children: ReactNode }) => {
    return <div className="pt-2">{props.children}</div>
  },

  Title: (props: { children: ReactNode }) => {
    return <h1 className="font-bold text-4xl">{props.children}</h1>
  },

  Subtitle: (props: { children: ReactNode }) => {
    return <h2 className="font-bold text-2xl">{props.children}</h2>
  },

  Subtext: (props: { children: ReactNode }) => {
    return <p className="text-lg text-muted-foreground">{props.children}</p>
  },
}
