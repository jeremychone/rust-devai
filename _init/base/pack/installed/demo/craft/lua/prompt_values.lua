local text_prompt_template = [[
Enter your text to be proofread.
Alternatively, add a `====` line separator to provide instructions before the separator and the content to which the instructions apply after the `====`.
]]

local code_prompt_template = [[
Ask your coding question to generate code below the `====`
And (optionally) add the code below the `====` as a starting point
]]

return {
  text_prompt_template = text_prompt_template,
  code_prompt_template = code_prompt_template
}
