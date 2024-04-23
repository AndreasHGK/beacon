import type { Metadata } from "next"
import localFont from "next/font/local"
import "./globals.css"
import { cn } from "@/lib/utils"

const fontSans = localFont({
  src: "./fonts/inter.ttf",
  variable: "--font-sans",
})

export const metadata: Metadata = {
  title: "Beacon",
  description: "A file sharing service.",
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <body
        className={cn(
          "min-h-screen bg-background font-sans antialiased",
          fontSans.variable
        )}
      >
        {children}
      </body>
    </html>
  )
}
