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

pub static HX_PYTHON_TAGS: &str = r#"
(
	(comment) @hx_comment
)
"#;

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
