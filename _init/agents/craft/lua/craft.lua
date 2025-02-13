
-- Returns FileRecord
function prep_input_file(input, options) 
  options = options or {}
  local default_name = options.default_name or "_craft"
  local placeholder_suffix = options.placeholder_suffix or "Write your content"
  local add_separator = options.add_separator ~= nil and options.add_separator or false 

  -- Enter default file_stem
  if input == nil then
    input = default_name
  end

  -- Determine the path
  local path = nil
  if type(input) == "string" then
      path = utils.text.ensure(input, {prefix = "./", suffix = ".md"})
  else
      path = input.path
  end

  -- create if needed
  local first_time = utils.path.exists(path) ~= true
  local placeholder_content = "placeholder - " .. placeholder_suffix
  if add_separator then
    placeholder_content = placeholder_content .. " \n\n====\n\n"
  end
  utils.file.ensure_exists(path,placeholder_content)  

  -- open if first time
  if first_time then 
    utils.cmd.exec("code", {path})
  end

  return utils.file.load(path)
end

-- Will return a devai skip if this task should be skipped
--   - If both inst and content are empty
--   - Or if inst (or content if inst is empty) starts with 'placeholder'
function should_skip(inst, content) 
  inst = inst and utils.text.trim(inst) or ""
  content = content and utils.text.trim(content) or ""


  if inst == "" and content == "" then
    return devai.skip("Empty content and instructions - Start writing, and do a Redo.")
  end

  local first_part = (inst ~= "" and inst) or content

  -- if starts with placeholder
  if first_part:sub(1, 11):lower() == "placeholder" then
      return devai.skip("Content is a placeholder, so skipping for now")
  end 

  return nil
end

-- retuns `inst, content` and each can be nil
-- options {content_is_default = bool}
--   - When content_is_default, means that if no two parts, the content will be the first_part
function prep_inst_and_content(content, separator, options) 
  local content_is_default = options and options.content_is_default or false
  local first_part, second_part = utils.text.split_first(content, separator)

  local inst, content = nil, nil
  if second_part ~= nil then 
    inst = first_part
    content = second_part
  elseif content_is_default then
    content = first_part
  else 
    inst = first_part
  end
    
  return inst, content
end

return {
  prep_input_file = prep_input_file,
  should_skip = should_skip,
  prep_inst_and_content = prep_inst_and_content
}