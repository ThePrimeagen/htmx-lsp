/// Search for htmx attributes, annotated by @attr_name
/// `@attr_name` must start with `hx-`.
///
/// `@complete_match` is @attr_name
///
/// `@unfinished_tag` contains error in query but it can still recommend
/// htmx attribute.
pub static HX_NAME: &str = r#"
(
        [
            (_ 
                (tag_name) 

                (_)*

                (attribute (attribute_name) @attr_name) @complete_match

                (#eq? @attr_name @complete_match)
            )

            (_ 
              (tag_name) 

              (attribute (attribute_name)) 

             (ERROR)? @equal_error
            ) @unfinished_tag
        ]

        (#match? @attr_name "hx-.*")
)
    
"#;

/// Searches for position in completion.
///
/// `@attr_name` - attribute name.
///
/// `@attr_value` - attribute value.
///
/// `@open_quote_error` - started typing but second quote is missing.
///
/// `@error_char` - means missing '=' or badly positioned '='.
///
/// `@empty_attribute` - attribute value is empty, completion can start inside.
///
/// `@non_empty_attribute` - attribute value isn't empty, usually used for hover,
/// goto implementation and goto definition request.
pub static HX_VALUE: &str = r#"
(
        [
          (ERROR 
            (tag_name) 

            (attribute_name) @attr_name 
            (_)
          ) @open_quote_error

          (_ 
            (tag_name)

            (attribute 
              (attribute_name) @attr_name
              (_)
            ) @last_item

            (ERROR) @error_char
          )

          (_
            (tag_name)

            (attribute 
              (attribute_name) @attr_name
              (quoted_attribute_value) @quoted_attr_value

              (#eq? @quoted_attr_value "\"\"")
            ) @empty_attribute
          )

          (_
            (tag_name) 

            (attribute 
              (attribute_name) @attr_name
              (quoted_attribute_value (attribute_value) @attr_value)

              ) @non_empty_attribute 
          )
        ]

        (#match? @attr_name "hx-.*")
)"#;

/// Rust comments should be inside of function or closure.
/// `@hx_comment` - comment. Tag starts after '@'.
pub static HX_RUST_TAGS: &str = r#"
(
  [
    (
        (function_item
            (block
                (line_comment) @hx_comment
            )
        )

    )

    (
      (closure_expression
          (block
              (line_comment) @hx_comment
          )
      )
    )

  ]

	(#match? @hx_comment " hx@.*")
)
"#;

/// JavaScript/TypeScript comments are inside any type of function.
/// `@hx_comment` - comment. Tag starts after '@'.
pub static HX_JS_TAGS: &str = r#"
(
	[
      (function_declaration
          (statement_block
          	(comment) @hx_comment
          )
      ) 
      
      (arrow_function
        (statement_block
          (comment) @hx_comment
        )
      ) 

      (method_definition
        (statement_block
            (comment) @hx_comment
        )
      )
    ]
    
    (#match? @hx_comment " hx@")
)
"#;

/// Find hx-lsp attribute. Used for goto implementation.
pub static HX_HTML: &str = r#"
(
	(attribute
    	(attribute_name) @attr_name
        (quoted_attribute_value
        	(attribute_value) @attr_value
        	)
    ) @hx_comment
    
    (#match? @attr_name "hx-lsp")
)
"#;

/// Replace "NAME" with any html attribute. At the moment not used query.
pub static HX_ANY_HTML: &str = r#"
(
	(attribute
    	(attribute_name) @attr_name
        (quoted_attribute_value
        	(attribute_value) @attr_value
        	)
    )
    
    (#match? @attr_name "NAME")
)
"#;

/// Python comments work much more different than in other languages.
/// They can be everywhere. Because of identation it has no information about
/// scope.
/// `@hx_comment` - comment. Tag starts after '@'.
pub static HX_PYTHON_TAGS: &str = r#"
(
	(comment) @hx_comment
)
"#;

/// Go comments are inside of function.
/// `@hx_comment` - comment. Tag starts after '@'.
pub static HX_GO_TAGS: &str = r#"

(
	[
   		(function_declaration
        	(block
            	(comment) @hx_comment
            )
        )
    ]
    
	(#match? @hx_comment " hx@")
)
	    
"#;
