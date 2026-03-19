import * as React from "react"
import { cn } from "@/lib/utils"

const Slider = React.forwardRef<
  HTMLInputElement,
  React.InputHTMLAttributes<HTMLInputElement>
>(({ className, ...props }, ref) => (
  <input
    type="range"
    className={cn(
      "w-full h-2 bg-secondary rounded-lg appearance-none cursor-pointer",
      className
    )}
    ref={ref}
    {...props}
  />
))
Slider.displayName = "Slider"

export { Slider }
