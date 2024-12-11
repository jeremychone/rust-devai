
## Skip if a string only has whitepace

```lua
-- If the string contains only whitespace, treat it as empty and skip.
-- `%S` matches any non-whitespace character.
-- `find` returns 
--    - the start and end indices if a match is found;
--    - otherwise, it returns nil. Thus, `not ...` evaluates to true.
if not content:find("%S") then
    return devai.skip("Empty file - skipping for now. Start writing, and do a Redo.")
end
```
