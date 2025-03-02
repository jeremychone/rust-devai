local prompt_template = [[
```toml
#!meta - parametric agent block

# knowledge_globs = ["/some/path/to/knowledge.md"] # can be relative to base_dir, or absolute

# base_dir = "src"
# context_globs = ["**/*.rs", "**/*.lua", "**/*.go", "**/*.java", "**/*.html", "**/*.js", "**/*.ts", "**/*.tsx", "**/*.css", "**/*.pcss", "**/*.scss"]
# input_globs = ["**/mod.rs"]

# This will override what is defined in the config.toml(s)
model_aliases = {claude = "claude-3-7-sonnet-latest", high = "o3-mini-high", low = "o3-mini-low", fast = "gemini-2.0-flash"}

# If not set, will see the model defined in the ~/.aipack-base/config.toml or .aipack/config.toml
model = "low" # "claude" often the best, "low" runs code faster, "high" is better, "fast" super fast (goot for simple task).

# Try asking: "List agent parameters and explain them." to get the parameters documentation. 
```


====

> Ask your coding questions above the `====` delimiter, and press `r` in the terminal to replay.
> 
> `coder` Agent parameters supported for this `coder` agent: 
>
> `base_dir`      - If activated in the TOML parameter section above, the AI will read and update the appropriate file.
> 
> `context_globs` - Customize your context globs relative to `base_dir` to decide which files are added to the context. 
>                   If not defined, and `base_dir` is defined, the defaults are shown above.
>                   These files will be described to the AI as `User's context files`.
>                   Narrowing the scope is better (cost and quality-wise, as it allows the model to focus on what matters).
> 
> `input_globs`   - Customize your input globs so that each file in those globs will be run independently/concurrently if set up with `input_concurrency`.
>                   If not defined, no input files will be included in the prompt. The agent will run only once, and the context files will be used. 
>                   This is optional and should be used in addition to `context_globs`, as these will be the working files. 
>                   These files will be described to the AI as `User's input files`.
>                   Otherwise, `context_globs` will be the working files. 
> 
> `model_aliases` - You can create your own alias names (which will override those defined in `.aipack/config.toml`).
>                   Top coder: "o3-mini-high" (aliased to 'high'), Fastest/~Cheapest: "gemini-2.0-flash-001".
> 
> `model`         - Provide a direct model name or a model alias to specify which model this coder agent should use.
>
> Lines starting with `>` above the `====` or in the first lines just below the `====` will be ignored in the AI conversation.
> Here, give your instructions, questions, and more. By default, the code will be below.
> 
> Feel free to remove these `> ` lines as they are just for initial documentation and have no impact on AI instructions.
> 
> You can ask, "Can you explain the coder agent parameters?" to get some documentation about it. 
>
> Happy Coding!
]]

return {
  prompt_template = prompt_template
}
