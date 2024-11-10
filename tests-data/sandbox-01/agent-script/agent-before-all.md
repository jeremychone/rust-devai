# Config

```toml
[genai]
model = "test_model_for_demo"
```


# Before All

```rhai
let some_data = "Some Before All";
return some_data;
```

# Data

```rhai
let some_data = "Some Data";
return some_data;
```

# Output

```rhai

return before_all + " - "  + data + " - " + item.path;
```

