The hx-headers attribute allows you to add to the headers that will be submitted with an AJAX request.

By default, the value of this attribute is a list of name-expression values in JSON (JavaScript Object Notation) format.

If you wish for hx-headers to evaluate the values given, you can prefix the values with javascript: or js:.

```html
<div hx-get="/example" hx-headers='{"myHeader": "My Value"}'>Get Some HTML, Including A Custom Header in the Request</div>
```

Security Considerations

    By default, the value of hx-headers must be valid JSON. It is not dynamically computed. If you use the javascript: prefix, be aware that you are introducing security considerations, especially when dealing with user input such as query strings or user-generated content, which could introduce a Cross-Site Scripting (XSS) vulnerability.

Notes

    hx-headers is inherited and can be placed on a parent element.
    A child declaration of a header overrides a parent declaration.


[HTMX Reference](https://htmx.org/attributes/hx-headers/)
