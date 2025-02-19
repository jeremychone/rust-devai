local prompt_template = [[
Ask your coding question or instructions. Remove this placeholder line and provide the base_dir below. 

```toml
#!meta
# Provide the base_dir you want to work from (if absent, the response will be in the second part of this prompt file, and no file will be created/updated)
# base_dir = "src"

# Customize your globs from base_dir (see below default)
# Narrow is better (cost and quality, as it allows the model to focus on what matters)
# This will put your file in the `## User's context & source files` as code blocks. 
# context_globs = ["**/*.rs", "**/*.lua", "**/*.go", "**/*.java", "**/*.html", "**/*.js", "**/*.ts", "**/*.tsx", "**/*.css", "**/*.pcss", "**/*.scss"]

# Customize your nput globs, so that each file of those globs will be ran independently/concurrrently if setup with input_concurrency
# This is optional, and should be used on top of context_globs, as this will be the working files. Otehrwise, context_globs will be the working files. 
# This will put your file in the `## User's input files` as code blocks. 
# input_globs = ["**/mod.rs"]


# Here, set your model override (by default o3-mini-high)
# Top coder: "o3-mini-high", Fastest: "gemini-2.0-flash-001" 
# model = "gemini-2.0-flash-001"
```
]]

return {
  prompt_template = prompt_template
}