# Config

```toml
[genai]
model = "test_model_for_demo"
```


# Before All

```rhai
let some_data = "Some Before All Data";
return #{
    before_all_data: some_data
};
```

# Data

```rhai
let some_data = "Some Data";
return some_data;
```

# Output

```rhai
let some_output = "Some Output";

return before_all_data + " - " + some_output;
```

