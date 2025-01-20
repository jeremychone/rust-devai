
# Markers Instruction

> This will be notes that will be excluded in the `utils.file.read_md_section(...) -> String`

- In the code above, instruction sections are delimited by `<<START>>` and `<<END>>`. These will be called instruction sections.

- They may span multiple lines for a given instruction section.

- For each instruction section:
    - Provide the new content for that instruction section.
    - Return only the result of the instruction sections, in the same order.
    - Put the result in a markdown block with the language `ai-answer` in their respective order.

- If here is no marker, that's fine, do not answer anything, just "No markers here, nothing to do."

- Ensure the order of the instruction sections is respected.
