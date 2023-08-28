This extension disables an element during an htmx request, when configured on the element triggering the request.

Install
<script src="https://unpkg.com/htmx.org/dist/ext/disable-element.js"></script>

Usage
Nominal case: disabling the element triggering the request
<button hx-get="/whatever" hx-ext="disable-element" hx-disable-element="self">Click me</button>

Disabling another element
<button hx-get="/whatever" hx-ext="disable-element" hx-disable-element="#to-disable">Click me</button>
<button id="to-disable">Watch me being disabled</button>

[HTMX Reference](https://htmx.org/extensions/disable-element/)
