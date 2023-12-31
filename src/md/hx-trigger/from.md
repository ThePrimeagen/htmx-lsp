from:<Extended CSS selector> - allows the event that triggers a request to come from another element in the document (e.g. listening to a key event on the body, to support hot keys)

A standard CSS selector resolves to all elements matching that selector.
Thus, `from:input` would listen to every input on the page.

The extended CSS selector here allows for the following non-standard CSS values:

* `document` - listen for events on the document
* `window` - listen for events on the window
* `closest <CSS selector>` - finds the closest ancestor element or itself, matching the given css selector
* `find <CSS selector>` - finds the closest child matching the given css selector


[HTMX Reference](https://htmx.org/attributes/hx-trigger/)
