# .gbtheme UI Theming

The .gbtheme package provides visual customization for your bot's web interface through straightforward CSS-based theming. This approach prioritizes simplicity—you create CSS files that override default styles, without needing complex build tools, template engines, or JavaScript frameworks.

## The Philosophy of Simple Theming

Many theming systems require elaborate toolchains, preprocessors, and build processes that create barriers for non-developers who want to customize their bot's appearance. General Bots takes a different approach by using plain CSS files that any web designer can create and modify.

This simplicity doesn't sacrifice capability. CSS custom properties (variables) provide the flexibility to change colors, typography, spacing, and other visual characteristics throughout the interface by modifying a few central values. The bot's default interface handles all the complex layout and functionality concerns, leaving themes to focus purely on appearance.

## Theme Structure

A theme consists of one or more CSS files placed in the .gbtheme folder within your bot package. The simplest theme might be a single default.css file containing variable overrides. More complex setups might include multiple theme files for different contexts—a dark theme for evening use, a high-contrast theme for accessibility, or seasonal themes for special occasions.

The system automatically loads the default theme on startup, and scripts can switch between available themes at runtime based on user preferences, time of day, or any other logic you implement.

## CSS Variables and Customization

The bot interface defines a set of CSS custom properties that control fundamental visual characteristics. By overriding these properties in your theme file, you can transform the interface's appearance with minimal code.

The primary-color variable establishes your main brand color, used for headers, buttons, and other prominent elements. The secondary-color provides accent coloring for highlights and interactive elements. Background and text-color control the basic page appearance and readability.

Typography settings including font-family let you match your organization's brand standards. Structural properties like border-radius affect the overall feel—sharp corners suggest professionalism while rounded corners feel friendlier. Spacing controls help maintain consistent visual rhythm throughout the interface.

These variables cascade through the interface components, meaning a single change propagates everywhere that property is used. This approach makes comprehensive theming achievable with just a handful of variable overrides.

## Creating Effective Themes

Building a theme starts with understanding your visual goals. Corporate deployments often need to match existing brand guidelines, requiring specific colors, fonts, and visual treatments. Consumer-facing bots might prioritize approachability and visual appeal. Internal tools might emphasize clarity and efficiency over aesthetics.

A minimal theme might override only the primary and secondary colors to match brand standards while accepting defaults for everything else. This approach gets results quickly with minimal effort. As needs grow, you can progressively add more customization.

When creating dark themes, remember to adjust not just the background color but also text colors, borders, shadows, and any other elements that assume a light background. Contrast matters for readability—test your themes with actual content to ensure text remains legible.

Accessibility considerations should inform theme design. Ensure sufficient contrast ratios between text and backgrounds, avoid relying solely on color to convey information, and test with various visual impairments in mind.

## Dynamic Theme Switching

Bots can change themes at runtime through the CHANGE THEME keyword in BASIC scripts. This capability enables several useful patterns.

User preference systems let visitors choose their preferred theme, with the selection stored in user memory for future visits. Time-based switching can apply dark themes during evening hours automatically. Contextual theming might use different visual treatments for different conversation modes or topics.

Theme switching happens instantly without page reloads, providing smooth transitions that maintain conversation flow.

## Configuration Integration

Theme settings can also be specified in the bot's config.csv file, providing default values that themes can override. The theme parameter specifies which theme file to load by default. The theme-color1 and theme-color2 parameters provide primary and secondary colors that the interface uses when no theme file specifies otherwise.

These configuration values serve as fallbacks—CSS files in the .gbtheme folder take precedence when they define the same properties. This layering allows simple color customization through configuration while supporting full CSS theming for more sophisticated needs.

## No Build Process Required

Unlike many modern web development workflows, .gbtheme requires no build tools, preprocessors, or compilation steps. You write CSS files, place them in the appropriate folder, and they take effect. Changes appear immediately through hot reloading, making iterative design work fast and responsive.

This simplicity means designers without development environment setup can contribute themes. Anyone who can write CSS can customize the interface, lowering barriers to visual customization.

## Migrating from Complex Systems

Organizations moving from platforms with complex theming systems can extract their essential visual parameters and recreate them as CSS variable overrides. The process typically involves identifying brand colors and typography from the existing theme, mapping those values to General Bots CSS variables, testing the result against the interface, and iteratively refining until the appearance matches expectations.

Much of the complexity in traditional theming systems exists to handle layout and functionality concerns that General Bots manages through its default interface. By focusing themes purely on visual styling, the migration process becomes much simpler.

## Best Practices

Effective theming follows several principles. Keep theme files focused and minimal—override only what you need to change rather than redefining everything. Start with a single default.css file and add complexity only as requirements demand.

Test themes across different devices and screen sizes to ensure they work well everywhere. Pay attention to interactive states like hover, focus, and active to ensure the interface remains usable and visually coherent.

Document theme choices, especially when values differ from brand guidelines for technical reasons. Future maintainers will appreciate understanding why specific decisions were made.

Maintain consistency within themes—if you override one color, consider whether related elements need adjustment to maintain visual harmony.

## Summary

The .gbtheme system demonstrates that powerful customization doesn't require complex tooling. Through CSS variables and standard stylesheets, you can transform the bot interface's appearance while the platform handles the underlying complexity. This approach respects the skills of designers and developers alike, enabling visual customization without artificial barriers.