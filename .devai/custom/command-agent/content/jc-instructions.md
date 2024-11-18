
# Markers Instruction

> This will be notes that will be excluded in the `file::read_md_section(...) -> String`

- In the code above, instruction sections are delimited by `<<START>>` and `<<END>>`. These will be called instruction sections.

- They may span multiple lines for a given instruction section.

- For each instruction section:
    - Provide the new content for that instruction section.
    - Return only the result of the instruction sections, in the same order.
    - Put the result in a markdown block with the language `ai-answer` in their respective order.

- If here is no marker, that's fine, do not answer anything, just "No markers here, nothing to do."

- Ensure the order of the instruction sections is respected.

# Rhai Doc Instruction

When asked to update or add documentation to a Rhai Doc. The two following parts need to be updated. 

First, in the the module comment (i.e. `//!`) section, make sure you update the `### Functions` part with all of the rhai functions like: 

```rust
//! ### Functions
//! * `html::prune_to_content(html_content: string) -> string`
```

Remove the one that are not in the file. 

Then, on top of each of the rust functions that is mapped to one or more rhai module function, update to follow the following format: 

```rust
/// ## RHAI Documentation
///
/// Perform a http GET request to the specified URL and returns an response object contain `.content` for it's text content.
///
/// ```
/// // API Signature
/// web::get(url: string) -> WebGetResponse (throws: WebGetException)
/// ```
///
/// By default, it will follows up to 5 redirects.
///
/// > Note: For now, only support text based content type.
///
///
/// ### Example
/// ```
/// let response = web::get("https://britesnow.com/test/text-page.txt")
/// let content = reponse.content;
/// ```
///
/// ### Returns (WebGetResponse)
///
/// Returns when the http response status code is 2xx range (will follow up to 5 redirects).
///
/// ```
/// {
///   success: true,    // true when the "final" http request is successful (2xx range)
///   status:  number,  // The status code returned by the http request
///   url:     string,  // The full URL requested
///   content: string,  // The text content
/// }
/// ```
///
/// ### Exception (WebGetException)
///
/// ```
/// {
///   success: false,   // false when the HTTP request is not successful
///   status?: number,  // (optional) The status code returned by the HTTP request
///   url:     string,  // The full URL requested
///   error:   string,  // The error message
/// }
/// ```
fn get(url: &str) -> RhaiResult {
	....
}
```

