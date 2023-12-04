The hx-disabled-elt attribute allows you to specify elements that will have the disabled attribute added to them for the duration of the request.

The value of this attribute is a CSS query selector of the element or elements to apply the class to, or the keyword closest, followed by a CSS selector, which will find the closest ancestor element or itself, that matches the given CSS selector (e.g. closest tr), or the keyword this.

Here is an example with a button that will disable itself during a request:

<button hx-post="/example" hx-disabled-elt="this">
    Post It!
</button>

When a request is in flight, this will cause the button to be marked with the disabled attribute, which will prevent further clicks from occurring.

Notes

    hx-disable-elt is inherited and can be placed on a parent element

[HTMX reference](https://htmx.org/attributes/hx-disabled-elt/)
