import { useTheme } from "next-themes"
import { Toaster as Sonner, ToasterProps } from "sonner"

const Toaster = ({ ...props }: ToasterProps) => {
  const { theme = "system" } = useTheme()

  return (
    <Sonner
      theme={theme as ToasterProps["theme"]}
      className="toaster group"
      position="bottom-left"
      visibleToasts={1}
      toastOptions={{
        classNames: {
          error: "group error !border-destructive !bg-destructive-half",
          success: "group success !border-primary/40",
          icon: "group-[.error]:!text-red-600 group-[.success]:!text-primary",
          title: "group-[.error]:!text-red-300"
        }
      }}
      style={
        {
          "--normal-bg": "var(--popover)",
          "--normal-text": "var(--popover-foreground)",
          "--normal-border": "var(--border)",
        } as React.CSSProperties
      }
      {...props}
    />
  )
}

export { Toaster }
